import asyncio

import typer
from loguru import logger
from typer_di import Depends, TyperDI

import nerve
from nerve.cli.utils import _get_run_args
from nerve.models import Configuration, Mode, Workflow
from nerve.runtime import logging, state
from nerve.runtime.agent import Agent
from nerve.runtime.flow import Flow
from nerve.runtime.runner import Arguments

cli = TyperDI(
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
    args: Arguments = Depends(_get_run_args),
) -> None:
    logging.init(
        args.log_path,
        level="DEBUG" if args.debug else "SUCCESS" if args.quiet else "INFO",
        litellm_debug=args.litellm_debug,
        litellm_tracing=args.litellm_tracing,
    )
    logger.info(f"🧠 nerve v{nerve.__version__}")

    asyncio.run(
        _run(
            ctx.args,
            args,
        )
    )


def _get_start_state_from_args(args: list[str]) -> dict[str, str]:
    # any unknown argument will populate the start_state
    start_state = {}
    for i in range(0, len(args), 2):
        key = args[i].removeprefix("--").removeprefix("-").replace("-", "_")
        value = args[i + 1] if i + 1 < len(args) else ""
        start_state[key] = value
    return start_state


async def _run(extra_args: list[str], args: Arguments) -> None:
    if args.trace:
        state.set_trace_file(args.trace)

    if args.interactive:
        state.set_mode(Mode.INTERACTIVE)

    # make variables available to the runtime
    start_state = args.start_state
    start_state.update(_get_start_state_from_args(extra_args))
    state.update_variables(start_state)

    # check if input_path is a workflow or single agent
    if Workflow.is_workflow(args.input_path):
        # full workflow
        flow = await Flow.from_path(
            args.input_path,
            window_strategy=args.conversation_strategy,
            max_steps=args.max_steps,
            max_cost=args.max_cost,
            timeout=args.timeout,
            start_state=start_state,
        )

    elif Configuration.is_agent_config(args.input_path):
        # single agent
        flow = await Flow.build(
            actors=[await Agent.create_from_file(args.generator, args.input_path, args.conversation_strategy)],
            max_steps=args.max_steps,
            max_cost=args.max_cost,
            timeout=args.timeout,
            start_state=start_state,
        )

    else:
        logger.error(f"path '{args.input_path}' is not a valid workflow or agent configuration")
        raise typer.Abort()

    await flow.run(args.task)

    logger.debug("exiting")
