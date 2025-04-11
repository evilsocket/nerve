import json
import pathlib
import typing as t

import typer
from loguru import logger

from nerve.defaults import (
    DEFAULT_AGENTS_LOAD_PATH,
    DEFAULT_CONVERSATION_STRATEGY,
    DEFAULT_GENERATOR,
    DEFAULT_MAX_COST,
    DEFAULT_MAX_STEPS,
    DEFAULT_TIMEOUT,
)
from nerve.generation import conversation
from nerve.runtime.runner import Arguments


def _resolve_input_path(input_path: pathlib.Path) -> pathlib.Path:
    # check if input_path exists
    if not input_path.exists():
        input_path_with_yaml = input_path.with_suffix(".yml")
        if input_path_with_yaml.exists():
            input_path = input_path_with_yaml

        elif not input_path.is_absolute():
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


def _get_run_args(
    input_path: t.Annotated[
        pathlib.Path,
        typer.Argument(help="Agent or workflow to execute"),
    ] = pathlib.Path("."),
    task: t.Annotated[
        str | None,
        typer.Option("--task", "-t", help="Set or override the task for the agent."),
    ] = None,
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
    litellm_debug: t.Annotated[
        bool,
        typer.Option("--litellm-debug", help="Enable litellm debug logging"),
    ] = False,
    litellm_tracing: t.Annotated[
        str | None,
        typer.Option("--litellm-tracing", help="Set litellm callbacks for tracing"),
    ] = None,
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
    trace: t.Annotated[
        pathlib.Path | None,
        typer.Option("--trace", help="Save the final state to a file."),
    ] = None,
    start_state: t.Annotated[
        str,
        typer.Option("--start-state", help="Pass the initial input state as a JSON string."),
    ] = "{}",
) -> Arguments:
    return Arguments(
        input_path=_resolve_input_path(input_path),
        task=task,
        generator=generator,
        # convert the conversation strategy string to a valid enum
        conversation_strategy_string=conversation_strategy,
        conversation_strategy=conversation.strategy_from_string(conversation_strategy),
        interactive=interactive,
        debug=debug,
        litellm_debug=litellm_debug,
        litellm_tracing=litellm_tracing,
        quiet=quiet,
        max_steps=max_steps,
        max_cost=max_cost,
        timeout=timeout,
        log_path=log_path,
        trace=trace,
        # parse the start_state JSON string into a dictionary
        start_state=json.loads(start_state),
    )
