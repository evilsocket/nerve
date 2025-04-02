import os
import typing as t
from contextlib import AsyncExitStack, asynccontextmanager

from loguru import logger
from mcp import ClientSession, StdioServerParameters, Tool
from mcp.client.stdio import stdio_client
from mcp.types import EmbeddedResource, ImageContent, TextContent

from nerve.models import Configuration


class Client:
    name: str = ""

    def __init__(self, name: str, server: Configuration.MCPServer):
        for key, value in server.env.items():
            if not value:
                env_val = os.getenv(key)
                if env_val:
                    logger.debug("setting {} from env", key)
                    server.env[key] = env_val

        self.name = name
        self.server = server
        self.server_params = StdioServerParameters(command=server.command, args=server.args, env=server.env)
        self._session: ClientSession | None = None
        self._exit_stack = AsyncExitStack()
        self._tools: list[Tool] = []

    @asynccontextmanager
    async def _create_streams(
        self,
    ) -> t.AsyncGenerator[tuple[t.Any, t.Any], None]:
        try:
            async with stdio_client(server=self.server_params) as (read_stream, write_stream):
                try:
                    yield read_stream, write_stream
                except Exception as e:
                    logger.debug("error yielding streams: {}", e)
        except Exception as e:
            # TODO: there's a weird bug, if we don't do this when the process exits
            # we will see an exception
            logger.debug("error creating streams: {}", e)
            exit(0)

    async def connect(self) -> None:
        if self._session:
            return

        logger.debug("connecting to MCP server {}: {}", self.name, self.server_params)

        self._read_stream, self._write_stream = await self._exit_stack.enter_async_context(self._create_streams())
        self._session = await self._exit_stack.enter_async_context(
            ClientSession(read_stream=self._read_stream, write_stream=self._write_stream)
        )

        await self._session.initialize()

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

        # TODO: this is paginated, fetch all pages
        response = await self._session.list_tools()
        self._tools = response.tools

        return self._tools

    async def call_tool(self, mcp_tool_name: str, **kwargs: t.Any) -> t.Any:
        if not self._session:
            await self.connect()
            if not self._session:
                raise Exception("failed to connect to MCP server")

        logger.debug("calling mcp tool: {}", mcp_tool_name)
        ret = await self._session.call_tool(mcp_tool_name, kwargs)
        logger.debug("mcp tool call result: {}", ret)

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
