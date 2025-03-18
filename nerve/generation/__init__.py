import asyncio
import json
import os
import typing as t
from abc import ABC, abstractmethod

from loguru import logger

from nerve.models import Usage
from nerve.runtime import state
from nerve.tools.protocol import get_tool_response, get_tool_schema


class WindowStrategy(ABC):
    @abstractmethod
    async def get_window(self, history: list[dict[str, t.Any]]) -> list[dict[str, t.Any]]:
        pass

    @abstractmethod
    def __str__(self) -> str:
        pass


class Engine(ABC):
    def __init__(
        self,
        generator_id: str,
        window_strategy: WindowStrategy,
        tools: list[t.Callable[..., t.Any]] | None = None,
    ):
        self.generator_id = generator_id
        self.generator_params: dict[str, t.Any] = {}

        self._parse_generator_params()

        self.history: list[dict[str, t.Any]] = []
        self.window_strategy = window_strategy

        self.tools = {fn.__name__: fn for fn in (tools or [])}
        self.tools_schemas = []
        for tool_name, tool_fn in self.tools.items():
            if not tool_fn.__doc__:
                raise ValueError(f"Tool {tool_name} has no docstring")
            self.tools_schemas.append(get_tool_schema(tool_fn))

    def _parse_generator_params(self) -> None:
        if "?" in self.generator_id:
            # split generator_id by '?' and parse the right part as query parameters
            base_id, query_params = self.generator_id.split("?", 1)
            self.generator_id = base_id
            # parse as parameters
            for param in query_params.split("&"):
                key, value = param.split("=", 1)
                # try to convert value to float if it looks like a number
                try:
                    if "." in value:
                        self.generator_params[key] = float(value)
                    elif value.isdigit():
                        self.generator_params[key] = int(value)
                    else:
                        self.generator_params[key] = value
                except Exception:
                    # if conversion fails, keep as string
                    self.generator_params[key] = value

        # set API base from parameters or environment variable
        if "api_base" in self.generator_params:
            self.api_base = self.generator_params["api_base"]
            del self.generator_params["api_base"]
        elif "GENERATOR_API_BASE" in os.environ:
            self.api_base = os.environ["GENERATOR_API_BASE"]
        else:
            self.api_base = None

    def _get_extended_tooling_schema(self, extra_tools: dict[str, t.Callable[..., t.Any]]) -> list[dict[str, t.Any]]:
        tools_schemas = self.tools_schemas.copy()
        extra_schemas = []

        for tool_name, tool_fn in extra_tools.items():
            if tool_name in self.tools:
                raise ValueError(f"Tool {tool_name} already exists")
            else:
                logger.debug(f"adding extra tool: {tool_name} / {tool_fn.__name__}")
                extra_schemas.append(get_tool_schema(tool_fn))

        tools_schemas.extend(extra_schemas)

        logger.trace(tools_schemas)
        logger.debug(f"{len(tools_schemas)} tools")

        return tools_schemas

    def _get_text_response(self, content: str) -> dict[str, t.Any]:
        state.on_event(
            "text_response",
            {
                "generator": self.generator_id,
                "response": content,
            },
        )

        return {
            "role": "user",
            "content": "None of the tools were used, interact with the user by executing the existing tools.",
        }

    def _get_unknown_tool_response(self, tool_call_id: str, tool_name: str) -> dict[str, t.Any]:
        state.on_event(
            "unknown_tool",
            {
                "generator": self.generator_id,
                "tool_name": tool_name,
            },
        )

        return {
            "tool_call_id": tool_call_id,
            "role": "tool",
            "content": f"The tool {tool_name} is not available.",
        }

    async def _get_tool_response(
        self, tool_call_id: str, tool_name: str, tool_fn: t.Callable[..., t.Any], tool_args: dict[str, t.Any]
    ) -> list[dict[str, t.Any]]:
        logger.debug(f"calling tool: {tool_name} with args: {tool_args}")
        try:
            tool_response = tool_fn(**tool_args)
            # check if the tool function returned a coroutine
            if asyncio.iscoroutine(tool_response):
                tool_response = await tool_response
        except Exception as e:
            state.on_event(
                "tool_error",
                {
                    "generator": self.generator_id,
                    "tool_name": tool_name,
                    "args": tool_args,
                    "error": e,
                },
            )
            tool_response = f"ERROR while executing tool {tool_name}: {e}"

        generated_response = get_tool_response(tool_response)
        if isinstance(generated_response, str):
            return [
                {
                    "tool_call_id": tool_call_id,
                    "role": "tool",
                    "name": tool_name,
                    "content": generated_response,
                }
            ]
        else:
            """
            Handle:

                Invalid 'messages[3]'. Image URLs are only allowed for messages with role 'user', but this message with role 'tool' contains an image URL.", 'type': 'invalid_request_error', 'param': 'messages[3]', 'code': 'invalid_value'}}

            And:

                An assistant message with 'tool_calls' must be followed by tool messages responding to each 'tool_call_id'.
            """
            return [
                {
                    "tool_call_id": tool_call_id,
                    "role": "tool",
                    "name": tool_name,
                    "content": "",
                },
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": f"{tool_name} returned the following response:",
                        },
                        generated_response,
                    ],
                },
            ]

    async def _process_tool_call(
        self, call_id: str, tool_name: str, args: str | dict[str, t.Any], extra_tools: dict[str, t.Callable[..., t.Any]]
    ) -> list[dict[str, t.Any]]:
        # resolve tool
        tool_fn = self.tools.get(tool_name, extra_tools.get(tool_name, None))
        if tool_fn is None:
            # unknown tool
            return [self._get_unknown_tool_response(call_id, tool_name)]
        else:
            tool_call_args = json.loads(args) if isinstance(args, str) else args
            # execute tool and collect response
            return await self._get_tool_response(call_id, tool_name, tool_fn, tool_call_args)

    @abstractmethod
    async def step(
        self,
        system_prompt: str | None,
        user_prompt: str,
        extra_tools: dict[str, t.Callable[..., t.Any]] | None = None,
        extra_message: str | None = None,
    ) -> Usage:
        pass
