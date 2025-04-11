import asyncio
import json
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


def _convert_to_serializable(obj: t.Any) -> t.Any:
    if hasattr(obj, "model_dump"):
        return obj.model_dump()
    elif hasattr(obj, "__dict__"):
        result = {}
        for key, value in obj.__dict__.items():
            if not key.startswith("_"):  # Skip private attributes
                result[key] = _convert_to_serializable(value)
        return result
    elif isinstance(obj, dict):
        return {key: _convert_to_serializable(value) for key, value in obj.items()}
    elif isinstance(obj, list | tuple):
        return [_convert_to_serializable(item) for item in obj]
    else:
        return obj


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
                logger.warning(
                    f"model {self.generator_id} does not support function calling or not listed in litellm database"
                )
        else:
            self._ollama = OllamaGlue(self.api_base, self.generator_id, self.generator_params)

    async def _litellm_generate(
        self, conversation: list[dict[str, t.Any]], tools_schema: list[dict[str, t.Any]] | None
    ) -> tuple[Usage, t.Any]:
        try:
            logger.debug(f"litellm.api_base: {self.api_base}")
            logger.debug(f"litellm.conversation: {json.dumps(conversation, indent=2)}")

            # litellm.set_verbose = True
            response = litellm.completion(
                model=self.generator_id,
                messages=conversation,
                tools=tools_schema,
                tool_choice="auto" if tools_schema else None,
                verbose=False,
                api_base=self.api_base,
                **self.generator_params,
            )

            logger.debug(f"litellm.response: {response}")

            return Usage(
                prompt_tokens=response.usage.prompt_tokens,
                completion_tokens=response.usage.completion_tokens,
                total_tokens=response.usage.total_tokens,
                cost=response._hidden_params.get("response_cost", None),
            ), response.choices[0].message
        except litellm.RateLimitError as e:  # type: ignore
            logger.warning(f"rate limit exceeded, sleeping for 5 seconds: {e}")
            await asyncio.sleep(5)
            return await self._litellm_generate(conversation, tools_schema)

    async def _generate(
        self, conversation: list[dict[str, t.Any]], tools_schema: list[dict[str, t.Any]] | None
    ) -> tuple[Usage, t.Any]:
        logger.debug(f"tools schema: {json.dumps(tools_schema, indent=2)}")
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
            # TODO: implement forced rate limit
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
        except litellm.AuthenticationError as e:  # type: ignore
            logger.error(e)
            exit(1)
        except litellm.NotFoundError as e:  # type: ignore
            logger.error(e)
            exit(1)
        except litellm.BadRequestError as e:  # type: ignore
            logger.error(e)
            # logger.error(f"{traceback.format_exc()}")
            exit(1)
        except litellm.APIError as e:  # type: ignore
            logger.error(e)
            logger.error(f"{traceback.format_exc()}")
            exit(1)
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
        # https://github.com/evilsocket/nerve/issues/41
        self.history.append(_convert_to_serializable(message))
        self.history.extend(responses)

        return usage
