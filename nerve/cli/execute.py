import pathlib

import typer
from loguru import logger

import nerve.runtime.state as state
from nerve.generation import WindowStrategy
from nerve.models import Configuration, Mode, Workflow
from nerve.runtime.agent import Agent
from nerve.runtime.flow import Flow

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
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
            # check if it exists as part of the $HOME/.nerve/ directory
            in_home = pathlib.Path.home() / ".nerve" / input_path
            if in_home.exists():
                input_path = in_home

        if not input_path.exists():
            logger.error(f"path '{input_path}' does not exist")
            raise typer.Abort()

    return input_path


async def execute_flow(
    input_path: pathlib.Path,
    generator: str,
    conv_window_strategy: WindowStrategy,
    start_state_args: list[str],
    max_steps: int = 100,
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
            timeout=timeout,
            start_state=start_state,
        )

    elif Configuration.is_agent_config(input_path):
        # single agent
        flow = Flow.build(
            actors=[Agent.create_from_file(generator, input_path, conv_window_strategy)],
            max_steps=max_steps,
            timeout=timeout,
            start_state=start_state,
        )

    else:
        logger.error(f"path '{input_path}' is not a valid workflow or agent configuration")
        raise typer.Abort()

    await flow.run()
