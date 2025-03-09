import json
import typing as t
import uuid

import litellm
from loguru import logger

from nerve.generation import Engine, Usage, WindowStrategy
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

        if not self.is_ollama and self.tools:
            if not litellm.supports_function_calling(model=self.generator_id):  # type: ignore
                logger.error(f"model {self.generator_id} does not support function calling")
                exit(1)

        if self.is_ollama:
            import ollama

            self.ollama_model = self.generator_id.split("/")[-1]
            self.ollama_client = ollama.AsyncClient(host=self.api_base)

            logger.debug(f"using ollama client for model {self.ollama_model}")

    async def _generate(
        self, conversation: list[dict[str, t.Any]], tooling: list[dict[str, t.Any]] | None
    ) -> tuple[Usage, t.Any]:
        if self.is_ollama:
            # https://github.com/BerriAI/litellm/issues/6353
            response = await self.ollama_client.chat(
                model=self.ollama_model,
                messages=conversation,
                tools=tooling,
                **self.generator_params,
            )
            return Usage(
                prompt_tokens=0,
                completion_tokens=0,
                total_tokens=0,
            ), response.message
        else:
            try:
                # litellm.set_verbose = True
                response = litellm.completion(
                    model=self.generator_id,
                    messages=conversation,
                    tools=tooling,
                    tool_choice="auto" if tooling else None,
                    api_base=self.api_base,
                    **self.generator_params,
                )

                return Usage(
                    prompt_tokens=response.usage.prompt_tokens,
                    completion_tokens=response.usage.completion_tokens,
                    total_tokens=response.usage.total_tokens,
                ), response.choices[0].message
            except litellm.AuthenticationError as e:  # type: ignore
                logger.error(e)
                exit(1)

    async def step(
        self,
        system_prompt: str | None,
        user_prompt: str,
        extra_tools: dict[str, t.Callable[..., t.Any]] | None = None,
    ) -> Usage:
        # system prompt and user prompt always included
        conversation = [{"role": "system", "content": system_prompt}] if system_prompt else []
        conversation.append({"role": "user", "content": user_prompt})
        conversation.extend(await self.window_strategy.get_window(self.history))

        # build json schema for available tools
        extra_tools = extra_tools or {}
        tooling = self._get_extended_tooling_schema(extra_tools) or None
        try:
            # get message
            usage, message = await self._generate(conversation, tooling)
        except Exception as e:
            logger.error(e)
            return Usage(
                prompt_tokens=0,
                completion_tokens=0,
                total_tokens=0,
            )

        responses: list[dict[str, t.Any]] = []
        if tooling and not message.tool_calls:
            # no tool calls
            responses = [self._get_text_response(str(message.content))]

        elif message.tool_calls:
            logger.debug(message.tool_calls)
            # for each tool call
            for tool_call in message.tool_calls:
                # resolve tool
                tool_call_id = tool_call.id if hasattr(tool_call, "id") else str(uuid.uuid4())
                tool_name = tool_call.function.name or ""
                tool_fn = self.tools.get(tool_name, extra_tools.get(tool_name, None))
                if tool_fn is None:
                    # unknown tool
                    responses.append(self._get_unknown_tool_response(tool_call_id, tool_name))
                else:
                    tool_call_args = (
                        json.loads(tool_call.function.arguments)
                        if isinstance(tool_call.function.arguments, str)
                        else tool_call.function.arguments
                    )
                    # execute tool and collect response
                    responses.extend(await self._get_tool_response(tool_call_id, tool_name, tool_fn, tool_call_args))

                if state.is_active_task_done():
                    # break early from multiple tool calls
                    logger.debug(f"task {self.generator_id} complete")
                    break

        # add tool call + per-call response messages
        self.history.append(message.__dict__)
        self.history.extend(responses)

        return usage
