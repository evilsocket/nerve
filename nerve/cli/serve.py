import asyncio
import json
import pathlib
import typing as t

import typer
from loguru import logger

import nerve
from nerve.cli.defaults import (
    DEFAULT_CONVERSATION_STRATEGY,
    DEFAULT_GENERATOR,
    DEFAULT_MAX_COST,
    DEFAULT_MAX_STEPS,
    DEFAULT_TIMEOUT,
)
from nerve.runtime import logging

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=True,
    help="Serve an agent as a REST API or MCP server.",
)
def serve(
    ctx: typer.Context,
    input_path: t.Annotated[
        pathlib.Path,
        typer.Argument(help="Agent or workflow to serve"),
    ] = pathlib.Path("."),
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
) -> None:
    logging.init(log_path, level="DEBUG" if debug else "INFO", litellm_debug=litellm_debug)
    logger.info(f"🧠 nerve v{nerve.__version__}")
