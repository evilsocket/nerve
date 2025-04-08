import asyncio
import json
import pathlib
import time
import typing as t

from termcolor import colored
import typer
from loguru import logger
from typer_di import Depends, TyperDI

import nerve
from nerve.cli.defaults import DEFAULT_EVAL_RUNS
from nerve.cli.utils import _get_run_args
from nerve.models import Configuration
from nerve.runtime import logging
from nerve.server.runner import Arguments, Output, Runner

cli = TyperDI(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


@cli.command(
    context_settings={"allow_extra_args": True, "ignore_unknown_options": True, "help_option_names": ["-h", "--help"]},
    help="Execute an agent or a workflow in evaluation mode.",
)
def eval(
    args: Arguments = Depends(_get_run_args),
    runs: t.Annotated[int, typer.Option("--runs", "-r", help="Number of runs per case.")] = DEFAULT_EVAL_RUNS,
    output: t.Annotated[pathlib.Path, typer.Option("--output", "-o", help="Path to save the output.")] = pathlib.Path(
        "eval.json"
    ),
) -> None:
    logging.init(
        args.log_path,
        level="DEBUG" if args.debug else "SUCCESS" if args.quiet else "INFO",
        litellm_debug=args.litellm_debug,
    )
    logger.info(f"🧠 nerve v{nerve.__version__}")

    # validate and collect inputs from the agent
    if not Configuration.is_agent_config(args.input_path):
        logger.error(f"path '{args.input_path}' is not a valid agent configuration")
        raise typer.Abort()

    cases_path = args.input_path / "cases"
    if not cases_path.exists():
        logger.error(f"cases path {cases_path} does not exist")
        raise typer.Abort()

    result = {
        "started_at": time.time(),
        "args": args.to_serializable(),
        "cases": {},
    }

    cases = sorted(cases_path.glob("*"))
    eval_name = colored(args.input_path.name, "green", attrs=["bold"])
    logger.info(f"📊 {args.generator} vs {eval_name} | cases: {len(cases)} | runs: {runs}")

    for case_path in cases:
        result["cases"][case_path.name] = {
            "started_at": time.time(),
            "runs": [],
        }

        for run in range(runs):
            logger.debug(f"running {case_path.name} ({run + 1}/{runs})")
            run_output = asyncio.run(_run_case(args, case_path))
            result["cases"][case_path.name]["runs"].append(run_output.model_dump())
            if run_output.task_success:
                logger.success(
                    f"   {eval_name} / {case_path.name} ({run + 1}/{runs}): {run_output.steps} steps, {run_output.time}s, {run_output.usage}"
                )
            else:
                logger.error(
                    f"   {eval_name} / {case_path.name} ({run + 1}/{runs}): {run_output.steps} steps, {run_output.time}s, {run_output.usage}"
                )

            break

        break

    logger.debug(f"evaluation results: {result}")

    result["finished_at"] = time.time()
    output.write_text(json.dumps(result))
    logger.info(f"evaluation results saved to {output}")


async def _run_case(args: Arguments, case_path: pathlib.Path) -> Output:
    return await Runner(
        args,
        {
            "CASE_NAME": case_path.name,
            "CASE_PATH": case_path.absolute().as_posix(),
        },
    ).run()
