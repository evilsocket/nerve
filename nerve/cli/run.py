import asyncio
import pathlib
import typing as t

import typer
from loguru import logger

import nerve
from nerve.cli.defaults import (
    DEFAULT_AGENTS_LOAD_PATH,
    DEFAULT_CONVERSATION_STRATEGY,
    DEFAULT_GENERATOR,
    DEFAULT_MAX_COST,
    DEFAULT_MAX_STEPS,
    DEFAULT_TIMEOUT,
)
from nerve.generation import WindowStrategy, conversation
from nerve.models import Configuration, Mode, Workflow
from nerve.runtime import logging, state
from nerve.runtime.agent import Agent
from nerve.runtime.flow import Flow

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


@cli.command(
    context_settings={"allow_extra_args": True, "ignore_unknown_options": True, "help_option_names": ["-h", "--help"]},
    help="Execute an agent or a workflow.",
)
def run(
    ctx: typer.Context,
    input_path: t.Annotated[
        pathlib.Path,
        typer.Argument(help="Agent or workflow to execute"),
    ] = pathlib.Path("."),
    generator: t.Annotated[
        str,
        typer.Option("--generator", "-g", help="If the agent generator field is not set, use this generator."),
    ] = DEFAULT_GENERATOR,
    conversation_strategy: t.Annotated[
        str,
        typer.Option("--conversation", "-c", help="Conversation strategy to use"),
    ] = DEFAULT_CONVERSATION_STRATEGY,
    interactive: t.Annotated[
        bool,
        typer.Option("--interactive", "-i", help="Interactive mode"),
    ] = False,
    debug: t.Annotated[
        bool,
        typer.Option("--debug", help="Enable debug logging"),
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
        typer.Option("--timeout", "-t", help="Timeout in seconds"),
    ] = DEFAULT_TIMEOUT,
    log_path: t.Annotated[
        pathlib.Path | None,
        typer.Option("--log", help="Log to a file."),
    ] = None,
    trace: t.Annotated[
        pathlib.Path | None,
        typer.Option("--trace", help="Save the final state to a file."),
    ] = None,
) -> None:
    logging.init(log_path, level="DEBUG" if debug else "SUCCESS" if quiet else "INFO")
    logger.info(f"🧠 nerve v{nerve.__version__}")

    asyncio.run(
        _run(
            input_path,
            generator,
            # convert the conversation strategy string to a valid enum
            conversation.strategy_from_string(conversation_strategy),
            ctx.args,
            max_steps,
            max_cost,
            timeout,
            interactive,
            trace,
        )
    )


def _get_start_state(args: list[str]) -> dict[str, str]:
    # any unknown argument will populate the start_state
    start_state = {}
    for i in range(0, len(args), 2):
        key = args[i].removeprefix("--").removeprefix("-").replace("-", "_")
        value = args[i + 1] if i + 1 < len(args) else ""
        start_state[key] = value
    return start_state


def _resolve_input_path(input_path: pathlib.Path) -> pathlib.Path:
    # check if input_path exists
    if not input_path.exists():
        if not input_path.is_absolute():
            # check if it exists as part of the $HOME/.nerve/agents directory
            in_home = DEFAULT_AGENTS_LOAD_PATH / input_path
            if in_home.exists():
                input_path = in_home

            in_home_with_yaml = in_home.with_suffix(".yml")
            if in_home_with_yaml.exists():
                input_path = in_home_with_yaml

        if not input_path.exists():
            logger.error(f"path '{input_path}' does not exist")
            raise typer.Abort()

    return input_path


async def _run(
    input_path: pathlib.Path,
    generator: str,
    conv_window_strategy: WindowStrategy,
    start_state_args: list[str],
    max_steps: int = 100,
    max_cost: float = 10.0,
    timeout: int | None = None,
    interactive: bool = False,
    trace: pathlib.Path | None = None,
) -> None:
    if trace:
        state.set_trace_file(trace)

    if interactive:
        state.set_mode(Mode.INTERACTIVE)

    # check if input_path exists
    input_path = _resolve_input_path(input_path)

    # make variables available to the runtime
    start_state = _get_start_state(start_state_args)
    state.update_variables(start_state)

    # check if input_path is a workflow or single agent
    if Workflow.is_workflow(input_path):
        # full workflow
        flow = Flow.from_path(
            input_path,
            window_strategy=conv_window_strategy,
            max_steps=max_steps,
            max_cost=max_cost,
            timeout=timeout,
            start_state=start_state,
        )

    elif Configuration.is_agent_config(input_path):
        # single agent
        flow = Flow.build(
            actors=[Agent.create_from_file(generator, input_path, conv_window_strategy)],
            max_steps=max_steps,
            max_cost=max_cost,
            timeout=timeout,
            start_state=start_state,
        )

    else:
        logger.error(f"path '{input_path}' is not a valid workflow or agent configuration")
        raise typer.Abort()

    await flow.run()
