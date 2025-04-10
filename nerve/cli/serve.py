import asyncio
import pathlib
import sys
import typing as t

import typer
from loguru import logger
from typer_di import Depends, TyperDI

import nerve
from nerve.cli.utils import _get_run_args
from nerve.defaults import (
    DEFAULT_SERVE_HOST,
    DEFAULT_SERVE_PORT,
)
from nerve.models import Configuration
from nerve.runtime import Runtime, logging
from nerve.runtime.runner import Arguments
from nerve.server.mcp import create_mcp_server, create_sse_app, serve_stdio_app
from nerve.server.rest import create_rest_api, serve_http_app

cli = TyperDI(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


def _get_agent_name(input_path: pathlib.Path) -> str:
    stem = input_path.stem
    if stem in ("agent", "workflow", "task"):
        return input_path.parent.name
    return stem


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=True,
    help="Serve an agent as a REST API or MCP server.",
)
def serve(
    run_args: Arguments = Depends(_get_run_args),
    host: t.Annotated[
        str,
        typer.Option("--host", "-h", help="Bind host to serve the agent on."),
    ] = DEFAULT_SERVE_HOST,
    port: t.Annotated[
        int,
        typer.Option("--port", "-p", help="Bind port to serve the agent on."),
    ] = DEFAULT_SERVE_PORT,
    mcp: t.Annotated[
        bool,
        typer.Option("--mcp", help="Start as MCP server."),
    ] = False,
    mcp_sse: t.Annotated[
        bool,
        typer.Option("--sse", help="Start as MCP server with SSE transport."),
    ] = False,
    serve_tools: t.Annotated[
        bool,
        typer.Option(
            "--tools",
            "-t",
            help="Serve tools as MCP servers. Automatically enabled if agent doesn't have a system prompt.",
        ),
    ] = False,
    tools_only: t.Annotated[
        bool,
        typer.Option("--tools-only", help="Serve tools only."),
    ] = False,
) -> None:
    # log to stderr instead of stdout if we're running as MCP server without SSE
    log_target = sys.stderr if mcp and not mcp_sse else sys.stdout
    logging.init(
        run_args.log_path,
        level="DEBUG" if run_args.debug else "INFO",
        litellm_debug=run_args.litellm_debug,
        target=log_target,
    )
    logger.info(f"🧠 nerve v{nerve.__version__}")

    asyncio.run(_serve(host, port, mcp, mcp_sse, serve_tools, tools_only, run_args))


async def _serve(
    host: str,
    port: int,
    mcp: bool,
    mcp_sse: bool,
    serve_tools: bool,
    tools_only: bool,
    run_args: Arguments,
) -> None:
    # validate and collect inputs from the agent
    if not Configuration.is_agent_config(run_args.input_path):
        logger.error(f"path '{run_args.input_path}' is not a valid agent configuration")
        raise typer.Abort()

    logger.debug(f"loading agent from {run_args.input_path}")
    config = Configuration.from_path(run_args.input_path)
    agent_name = _get_agent_name(run_args.input_path)
    logger.debug(f"agent {agent_name} loaded: {config.description}")

    inputs = config.get_inputs()
    logger.debug(f"creating endpoint for inputs: {inputs}")
    runtime: Runtime | None = None

    if tools_only or not config.system_prompt and not config.agent and not config.task:
        logger.info("🧰 tools-only mode")
        serve_tools = True
        tools_only = True
    elif serve_tools:
        logger.info("🧠 + 🧰 serving agent and tools")
    else:
        logger.info("🧠 serving agent")

    if serve_tools:
        # if we have to serve tools, we need to build the runtime
        runtime = await Runtime.build(
            working_dir=run_args.input_path if run_args.input_path.is_dir() else run_args.input_path.parent,
            name=agent_name,
            configuration=config,
            debug=run_args.debug,
        )

    if mcp or mcp_sse:
        # MCP server
        server = create_mcp_server(
            agent_name,
            config,
            run_args,
            inputs,
            runtime,
            serve_tools,
            tools_only,
        )

        if mcp_sse:
            # via SSE (http)
            app = create_sse_app(run_args.debug, server)

            await serve_http_app(app, agent_name, "sse", host, port, run_args.debug)

        else:
            # via stdout (as a process)
            await serve_stdio_app(server, agent_name)

    else:
        # start as REST API
        app = create_rest_api(
            run_args,
            inputs,
            config,
            runtime,
            serve_tools,
            tools_only,
        )

        await serve_http_app(app, agent_name, "http", host, port, run_args.debug)
