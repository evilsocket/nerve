import asyncio
import pathlib
import sys
import typing as t

import typer
from loguru import logger

import nerve
from nerve.cli.defaults import (
    DEFAULT_CONVERSATION_STRATEGY,
    DEFAULT_GENERATOR,
    DEFAULT_MAX_COST,
    DEFAULT_MAX_STEPS,
    DEFAULT_SERVE_HOST,
    DEFAULT_SERVE_PORT,
    DEFAULT_TIMEOUT,
)
from nerve.cli.utils import _resolve_input_path
from nerve.models import Configuration
from nerve.runtime import Runtime, logging
from nerve.server.mcp import create_mcp_server, create_sse_app, serve_stdio_app
from nerve.server.rest import create_rest_api, serve_http_app

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


def _get_agent_name(input_path: pathlib.Path) -> str:
    stem = input_path.stem
    if stem in ("agent", "workflow", "task"):
        return input_path.parent.name
    return stem


def _get_agent_with_inputs(input_path: pathlib.Path) -> tuple[pathlib.Path, str, Configuration, dict[str, t.Any]]:
    resolved_input_path = _resolve_input_path(input_path)
    if not Configuration.is_agent_config(resolved_input_path):
        logger.error(f"path '{input_path}' is not a valid agent configuration")
        raise typer.Abort()

    logger.debug(f"loading agent from {resolved_input_path}")
    config = Configuration.from_path(resolved_input_path)
    agent_name = _get_agent_name(resolved_input_path)
    logger.debug(f"agent {agent_name} loaded: {config.description}")

    inputs = config.get_inputs()
    logger.debug(f"creating endpoint for inputs: {inputs}")

    return resolved_input_path, agent_name, config, inputs


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=True,
    help="Serve an agent as a REST API or MCP server.",
)
def serve(
    input_path: t.Annotated[
        pathlib.Path,
        typer.Argument(help="Agent or workflow to serve"),
    ] = pathlib.Path("."),
    host: t.Annotated[
        str,
        typer.Option("--host", "-h", help="Bind host to serve the agent on."),
    ] = DEFAULT_SERVE_HOST,
    port: t.Annotated[
        int,
        typer.Option("--port", "-p", help="Bind port to serve the agent on."),
    ] = DEFAULT_SERVE_PORT,
    generator: t.Annotated[
        str,
        typer.Option("--generator", "-g", help="If the agent generator field is not set, use this generator."),
    ] = DEFAULT_GENERATOR,
    conversation_strategy: t.Annotated[
        str,
        typer.Option("--conversation", "-c", help="Conversation strategy to use"),
    ] = DEFAULT_CONVERSATION_STRATEGY,
    debug: t.Annotated[
        bool,
        typer.Option("--debug", help="Enable debug logging"),
    ] = False,
    litellm_debug: t.Annotated[
        bool,
        typer.Option("--litellm-debug", help="Enable litellm debug logging"),
    ] = False,
    quiet: t.Annotated[
        bool,
        typer.Option("--quiet", "-q", help="Quiet mode"),
    ] = False,
    max_steps: t.Annotated[
        int,
        typer.Option("--max-steps", "-s", help="Maximum number of steps. Set to 0 to disable."),
    ] = DEFAULT_MAX_STEPS,
    max_cost: t.Annotated[
        float,
        typer.Option(
            "--max-cost",
            help="If cost information is available, stop when the cost exceeds this value in USD. Set to 0 to disable.",
        ),
    ] = DEFAULT_MAX_COST,
    timeout: t.Annotated[
        int | None,
        typer.Option("--timeout", help="Timeout in seconds"),
    ] = DEFAULT_TIMEOUT,
    log_path: t.Annotated[
        pathlib.Path | None,
        typer.Option("--log", help="Log to a file."),
    ] = None,
    mcp: t.Annotated[
        bool,
        typer.Option("--mcp", help="Start as MCP server."),
    ] = False,
    mcp_sse: t.Annotated[
        bool,
        typer.Option("--mcp-sse", help="Start as MCP server with SSE transport."),
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
    logging.init(log_path, level="DEBUG" if debug else "INFO", litellm_debug=litellm_debug, target=log_target)
    logger.info(f"🧠 nerve v{nerve.__version__}")

    asyncio.run(
        _serve(
            input_path,
            host,
            port,
            debug,
            mcp,
            mcp_sse,
            serve_tools,
            tools_only,
            generator,
            conversation_strategy,
            max_steps,
            max_cost,
            timeout,
            quiet,
        )
    )


async def _serve(
    input_path: pathlib.Path,
    host: str,
    port: int,
    debug: bool,
    mcp: bool,
    mcp_sse: bool,
    serve_tools: bool,
    tools_only: bool,
    generator: str,
    conversation_strategy: str,
    max_steps: int,
    max_cost: float,
    timeout: int | None,
    quiet: bool,
) -> None:
    # validate and collect inputs from the agent
    input_path, agent_name, config, inputs = _get_agent_with_inputs(input_path)
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
            working_dir=input_path if input_path.is_dir() else input_path.parent,
            name=agent_name,
            configuration=config,
            debug=debug,
        )

    if mcp or mcp_sse:
        # MCP server
        server = create_mcp_server(
            agent_name,
            config,
            inputs,
            input_path,
            generator,
            conversation_strategy,
            max_steps,
            max_cost,
            runtime,
            timeout,
            quiet,
            serve_tools,
            tools_only,
        )

        if mcp_sse:
            # via SSE (http)
            app = create_sse_app(debug, server)

            await serve_http_app(app, agent_name, "sse", host, port, debug)

        else:
            # via stdout (as a process)
            await serve_stdio_app(server, agent_name)

    else:
        # start as REST API
        app = create_rest_api(
            input_path,
            generator,
            conversation_strategy,
            max_steps,
            max_cost,
            timeout,
            quiet,
            inputs,
            config,
            runtime,
            serve_tools,
            tools_only,
        )

        await serve_http_app(app, agent_name, "http", host, port, debug)
