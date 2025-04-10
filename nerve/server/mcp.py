import asyncio
import json
import typing as t

import mcp.types as mcp_types
from fastapi import HTTPException
from loguru import logger
from mcp import stdio_server
from mcp.server import Server
from mcp.server.sse import SseServerTransport
from starlette.applications import Starlette
from starlette.routing import Mount, Route

from nerve.models import Configuration
from nerve.runtime import Runtime
from nerve.runtime.runner import Arguments, Output, Runner
from nerve.tools.protocol import get_tool_schema


def _get_input_state_from_request(inputs: dict[str, str | None], data: dict[str, str]) -> dict[str, str]:
    input_state: dict[str, str] = {}
    for input_name, default_value in inputs.items():
        # get user provided or default value if set
        input_value = data.get(input_name, default_value)
        if input_value is None:
            raise HTTPException(status_code=400, detail=f"input '{input_name}' is required")

        input_state[input_name] = input_value

    return input_state


def _create_listing_handler(
    tools: list[mcp_types.Tool],
) -> t.Callable[[t.Any], t.Coroutine[t.Any, t.Any, mcp_types.ServerResult]]:
    async def _handler(_: t.Any) -> mcp_types.ServerResult:
        return mcp_types.ServerResult(mcp_types.ListToolsResult(tools=tools))

    return _handler


async def _handle_agent_call(
    run_args: Arguments,
    inputs: dict[str, str | None],
    agent_name: str,
    mcp_server: Server,  # type: ignore
    arguments: dict[str, t.Any],
) -> Output:
    input_state = _get_input_state_from_request(inputs, arguments)
    runner = Runner(run_args, input_state)
    runner.set_stdout_fn(
        lambda x: asyncio.create_task(
            mcp_server.request_context.session.send_log_message(
                level="info",
                data=x,
                logger=agent_name,
            )
        )
    )
    runner.set_stderr_fn(
        lambda x: asyncio.create_task(
            mcp_server.request_context.session.send_log_message(
                level="error",
                data=x,
                logger=agent_name,
            )
        )
    )
    # execute the runner
    return await runner.run()


def _mcp_text_result(data: str, error: bool = False) -> mcp_types.ServerResult:
    return mcp_types.ServerResult(
        mcp_types.CallToolResult(content=[mcp_types.TextContent(type="text", text=data)], isError=error)
    )


def _mcp_error_result(error: Exception) -> mcp_types.ServerResult:
    return mcp_types.ServerResult(
        mcp_types.CallToolResult(content=[mcp_types.TextContent(type="text", text=str(error))], isError=True)
    )


def _create_call_handler(
    run_args: Arguments,
    inputs: dict[str, str | None],
    agent_name: str,
    mcp_server: Server,  # type: ignore
    runtime: Runtime | None,
) -> t.Callable[[mcp_types.CallToolRequest], t.Coroutine[t.Any, t.Any, mcp_types.ServerResult]]:
    tools_dict = {tool.__name__: tool for tool in runtime.tools} if runtime else {}

    async def _handler(req: mcp_types.CallToolRequest) -> mcp_types.ServerResult:
        nonlocal tools_dict

        try:
            arguments = req.params.arguments or {}
            tool = tools_dict.get(req.params.name, None)
            if tool:
                # execute a tool
                return _mcp_text_result(json.dumps(await tool(**arguments)))
            elif req.params.name == agent_name:
                # execute the agent itself
                call_result = await _handle_agent_call(
                    run_args,
                    inputs,
                    agent_name,
                    mcp_server,
                    arguments,
                )
                return _mcp_text_result(json.dumps(call_result.output))
            else:
                raise Exception(f"unknown tool: {req.params.name}")

        except Exception as e:
            return _mcp_error_result(e)

    return _handler


def create_mcp_server(
    agent_name: str,
    config: Configuration,
    run_args: Arguments,
    inputs: dict[str, str | None],
    runtime: Runtime | None,
    serve_tools: bool = False,
    tools_only: bool = False,
) -> Server:  # type: ignore
    server = Server(agent_name)  # type: ignore
    tools: list[mcp_types.Tool] = []

    # create a tool for the agent itself
    if not tools_only:
        logger.info(f"🧰 creating MCP tool for {agent_name}")
        agent_description = config.description or config.task
        tools.append(
            mcp_types.Tool(
                name=agent_name,
                description=agent_description,
                inputSchema={
                    "type": "object",
                    "required": [name for name, default_value in inputs.items() if default_value is None],
                    "description": agent_description,
                    "properties": {
                        name: {
                            "type": "string",
                            "description": "",  # TODO: anything we can add here?
                        }
                        for name in inputs.keys()
                    },
                },
            )
        )

    # create a tool for each tool in the runtime
    if serve_tools and runtime:
        logger.info(f"🧰 creating MCP tools for {len(runtime.tools)} agent tools")
        for tool in runtime.tools:
            input_schema = get_tool_schema(run_args.generator, tool).get("function", {}).get("parameters", {})
            logger.info(f"  {tool.__name__}")
            tools.append(
                mcp_types.Tool(
                    name=tool.__name__,
                    description=tool.__doc__,
                    inputSchema=input_schema,
                )
            )

    server.request_handlers[mcp_types.ListToolsRequest] = _create_listing_handler(tools)
    server.request_handlers[mcp_types.CallToolRequest] = _create_call_handler(
        run_args,
        inputs,
        agent_name,
        server,
        runtime,
    )

    return server


def create_sse_app(debug: bool, server: Server) -> Starlette:  # type: ignore
    sse = SseServerTransport("/messages/")

    async def handle_sse(request: t.Any) -> t.Any:
        async with sse.connect_sse(request.scope, request.receive, request._send) as streams:
            await server.run(streams[0], streams[1], server.create_initialization_options())

    return Starlette(
        debug=debug,
        routes=[
            Route("/sse", endpoint=handle_sse),
            Mount("/messages/", app=sse.handle_post_message),
        ],
    )


async def serve_stdio_app(server: Server, agent_name: str) -> None:  # type: ignore
    async with stdio_server() as streams:
        logger.info(f"🌐 serving {agent_name} on stdout ...")
        await server.run(streams[0], streams[1], server.create_initialization_options())
