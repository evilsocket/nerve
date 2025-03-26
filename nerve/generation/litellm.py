import traceback
import typing as t
import uuid

import litellm
from loguru import logger

from nerve.generation import Engine, WindowStrategy
from nerve.generation.conversation import SlidingWindowStrategy
from nerve.generation.ollama import OllamaGlue
from nerve.models import Usage
from nerve.runtime import state


class LiteLLMEngine(Engine):
    def __init__(
        self,
        generator_id: str,
        window_strategy: WindowStrategy,
        tools: list[t.Callable[..., t.Any]] | None = None,
    ):
        super().__init__(generator_id, window_strategy, tools)

        # until this is not fixed, ollama needs special treatment: https://github.com/BerriAI/litellm/issues/6353
        self.is_ollama = "ollama" in self.generator_id
        self.reduced_window_size = 25

        if not self.is_ollama:
            if self.tools and not litellm.supports_function_calling(model=self.generator_id):  # type: ignore
                logger.error(f"model {self.generator_id} does not support function calling")
                exit(1)

        else:
            self._ollama = OllamaGlue(self.api_base, self.generator_id, self.generator_params)

    async def _litellm_generate(
        self, conversation: list[dict[str, t.Any]], tools_schema: list[dict[str, t.Any]] | None
    ) -> tuple[Usage, t.Any]:
        try:
            # litellm.set_verbose = True
            response = litellm.completion(
                model=self.generator_id,
                messages=conversation,
                tools=tools_schema,
                tool_choice="auto" if tools_schema else None,
                api_base=self.api_base,
                **self.generator_params,
            )

            return Usage(
                prompt_tokens=response.usage.prompt_tokens,
                completion_tokens=response.usage.completion_tokens,
                total_tokens=response.usage.total_tokens,
                cost=response._hidden_params.get("response_cost", None),
            ), response.choices[0].message
        except litellm.AuthenticationError as e:  # type: ignore
            logger.error(e)
            exit(1)

    async def _generate(
        self, conversation: list[dict[str, t.Any]], tools_schema: list[dict[str, t.Any]] | None
    ) -> tuple[Usage, t.Any]:
        if self.is_ollama:
            # https://github.com/BerriAI/litellm/issues/6353
            return await self._ollama.generate(conversation, tools_schema)
        else:
            return await self._litellm_generate(conversation, tools_schema)

    async def _get_conversation(
        self, system_prompt: str | None, user_prompt: str, extra_message: str | None
    ) -> list[dict[str, t.Any]]:
        # @ system prompt and user prompt always included
        conversation = [{"role": "system", "content": system_prompt}] if system_prompt else []
        conversation.append({"role": "user", "content": user_prompt})
        conversation.extend(await self.window_strategy.get_window(self.history))

        if extra_message:
            conversation.append({"role": "user", "content": extra_message})

        logger.debug(f"{self.window_strategy} | conv size: {len(conversation)}")

        return conversation

    async def _generate_next_message(
        self,
        system_prompt: str | None,
        user_prompt: str,
        extra_tools: dict[str, t.Callable[..., t.Any]] | None = None,
        extra_message: str | None = None,
    ) -> tuple[Usage, t.Any]:
        # build chat history
        conversation = await self._get_conversation(system_prompt, user_prompt, extra_message)
        # build json schema for available tools
        extra_tools = extra_tools or {}
        tools_schema = self._get_extended_tooling_schema(extra_tools)

        try:
            # get next message
            return await self._generate(conversation, tools_schema)
        except litellm.ContextWindowExceededError as e:  # type: ignore
            logger.debug(e)
            if self.reduced_window_size > 0:
                self.reduced_window_size -= 1
                self.window_strategy = SlidingWindowStrategy(self.reduced_window_size)
                logger.error(f"context window exceeded, adjusting window strategy: {self.window_strategy}")
                return await self._generate_next_message(system_prompt, user_prompt, extra_tools)

            else:
                logger.error("context window exceeded")
                return Usage(
                    prompt_tokens=0,
                    completion_tokens=0,
                    total_tokens=0,
                ), None
        except Exception as e:
            logger.error(e)
            logger.error(f"{traceback.format_exc()}")
            return Usage(
                prompt_tokens=0,
                completion_tokens=0,
                total_tokens=0,
            ), None

    async def step(
        self,
        system_prompt: str | None,
        user_prompt: str,
        extra_tools: dict[str, t.Callable[..., t.Any]] | None = None,
        extra_message: str | None = None,
    ) -> Usage:
        extra_tools = extra_tools or {}

        # get next message
        usage, message = await self._generate_next_message(system_prompt, user_prompt, extra_tools, extra_message)
        if message is None:
            return usage

        # collect responses
        has_tools = len(extra_tools) > 0 or len(self.tools) > 0
        responses: list[dict[str, t.Any]] = []

        if has_tools and not message.tool_calls:
            # no tool calls, just return the text response
            responses = [self._get_text_response(str(message.content))]

        elif message.tool_calls:
            logger.debug(message.tool_calls)
            # for each tool call
            for tool_call in message.tool_calls:
                # resolve and execute the tool call
                responses.extend(
                    await self._process_tool_call(
                        tool_call.id if hasattr(tool_call, "id") else str(uuid.uuid4()),
                        tool_call.function.name or "",
                        tool_call.function.arguments,
                        extra_tools,
                    )
                )

                # break early from multiple tool calls if the task is complete
                if state.is_active_task_done():
                    logger.debug(f"task {self.generator_id} complete")
                    break

        # add tool call + per-call response messages
        self.history.append(message.__dict__)
        self.history.extend(responses)

        return usage
