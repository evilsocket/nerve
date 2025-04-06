import asyncio
import os
import pathlib
import sys
import typing as t
from contextlib import AsyncExitStack, asynccontextmanager

import mcp.types as mcp_types
from loguru import logger
from mcp import ClientSession, StdioServerParameters, Tool
from mcp.client.sse import sse_client
from mcp.client.stdio import stdio_client
from mcp.types import EmbeddedResource, ImageContent, TextContent

from nerve.models import Configuration


class Client:
    name: str = ""

    def __init__(self, name: str, server: Configuration.MCPServer, working_dir: pathlib.Path):
        for key, value in server.env.items():
            if not value:
                env_val = os.getenv(key)
                if env_val:
                    logger.debug("setting {} from env", key)
                    server.env[key] = env_val

            if not server.env[key]:
                logger.error("mcp server {} environment variable {} is not set", self.name, key)
                exit(1)

        # pass all env
        server.env.update(os.environ)

        self.name = name
        self.server = server
        self.server_params = StdioServerParameters(
            command=server.command, args=server.args, env=server.env, cwd=working_dir
        )
        self._session: ClientSession | None = None
        self._exit_stack = AsyncExitStack()
        self._tools: list[Tool] = []
        self._client_context: t.Any = None

    async def _logging_callback(
        self,
        params: mcp_types.LoggingMessageNotificationParams,
    ) -> None:
        line = str(params.data)
        # parts = line.split("]", 2)
        # line = parts[1].strip() if len(parts) > 1 else line  # remove timestamp
        # line = line.replace(str(params.level).upper(), "").strip()  # remove level
        getattr(logger, str(params.level))(f"<{self.name}> {line}")

    @asynccontextmanager
    async def _safe_stdio_context(
        self,
    ) -> t.AsyncGenerator[tuple[t.Any, t.Any], None]:
        gen_exit = False
        try:
            async with stdio_client(server=self.server_params, errlog=sys.stderr) as (read_stream, write_stream):
                logger.debug("stdio streams for {} created", self.name)
                try:
                    yield read_stream, write_stream
                except GeneratorExit:
                    logger.debug("GeneratorExit")
                    gen_exit = True
        except Exception:
            logger.debug(f"Exception, gen_exit = {gen_exit}")
            # TODO: there's a weird bug, if we don't do this when the process exits
            # we will see an exception
            if gen_exit:
                exit(0)

    async def connect(self) -> None:
        if self._session:
            return

        self._client_context = None
        if self.server.url:
            logger.debug("connecting to SSE MCP server {}", self.name)
            self._client_context = sse_client(
                url=self.server.url or "http://localhost:8080",
                headers=self.server.headers,
                timeout=self.server.timeout,
                sse_read_timeout=self.server.sse_read_timeout,
            )
        else:
            logger.debug("connecting to stdio MCP server {}", self.name)
            self._client_context = self._safe_stdio_context()

        self._read_stream, self._write_stream = await self._exit_stack.enter_async_context(self._client_context)
        logger.debug("creating async context for {}", self.name)
        self._session = await self._exit_stack.enter_async_context(
            ClientSession(
                read_stream=self._read_stream, write_stream=self._write_stream, logging_callback=self._logging_callback
            )
        )

        try:
            logger.debug("initializing session for {}", self.name)
            await asyncio.wait_for(self._session.initialize(), timeout=self.server.session_timeout)
        except asyncio.TimeoutError:
            logger.error(
                "mcp server {} initialization timeout",
                self.name,
            )
            exit(1)

        logger.debug("session initialized for {}", self.name)

        self._tools = await self.tools()

        logger.debug("connected to MCP server {} with {} tools", self.name, len(self._tools))
        for tool in self._tools:
            logger.debug("tool: {}", tool)

    async def tools(self) -> list[Tool]:
        if not self._session:
            await self.connect()
            if not self._session:
                raise Exception("failed to connect to MCP server")

        if self._tools:
            return self._tools

        logger.debug("listing tools from MCP server {}", self.name)

        response = await self._session.list_tools()
        logger.debug("list_tools.response: {}", response)
        self._tools = response.tools

        return self._tools

    async def call_tool(self, mcp_tool_name: str, **kwargs: t.Any) -> t.Any:
        if not self._session:
            await self.connect()
            if not self._session:
                raise Exception("failed to connect to MCP server")

        logger.debug("calling mcp tool {} with args: {}", mcp_tool_name, kwargs)
        ret = await self._session.call_tool(mcp_tool_name, kwargs)
        logger.debug("mcp {} call result: {}", mcp_tool_name, ret)

        if ret.isError:
            raise Exception(str(ret))

        responses: list[t.Any] = []
        for elem in ret.content:
            if isinstance(elem, TextContent):
                responses.append(elem.text)
            elif isinstance(elem, ImageContent):
                responses.append(
                    {
                        "type": "image_url",
                        "image_url": {"url": f"data:{elem.mimeType};base64,{elem.data}"},
                    }
                )
            elif isinstance(elem, EmbeddedResource):
                raise Exception("EmbeddedResource not supported yet")

        logger.debug("tool call responses: {}", responses)

        return responses[0] if len(responses) == 1 else responses
