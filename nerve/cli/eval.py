import asyncio
import pathlib
import typing as t

import typer
from loguru import logger
from termcolor import colored
from typer_di import Depends, TyperDI

import nerve
from nerve.cli.utils import _get_run_args
from nerve.defaults import DEFAULT_EVAL_RUNS
from nerve.models import Configuration
from nerve.runtime import logging
from nerve.runtime.eval import Case, Cases, Evaluation
from nerve.runtime.runner import Arguments, Output, Runner

cli = TyperDI(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


def _get_output_path(args: Arguments) -> pathlib.Path:
    output_name = f"{args.generator}-{args.input_path.name}"
    sanitized = ""
    for char in output_name:
        if char.isalnum() or char in "._- ":
            sanitized += char
        else:
            sanitized += "-"

    while "--" in sanitized:
        sanitized = sanitized.replace("--", "-")

    return pathlib.Path(f"{sanitized}.json")


@cli.command(
    context_settings={"allow_extra_args": True, "ignore_unknown_options": True, "help_option_names": ["-h", "--help"]},
    help="Execute an agent or a workflow in evaluation mode.",
)
def eval(
    args: Arguments = Depends(_get_run_args),
    runs: t.Annotated[int, typer.Option("--runs", "-r", help="Number of runs per case.")] = DEFAULT_EVAL_RUNS,
    output: t.Annotated[
        pathlib.Path | None,
        typer.Option("--output", "-o", help="Path to save the output. If not set will be auto generated."),
    ] = None,
) -> None:
    logging.init(
        args.log_path,
        level="DEBUG" if args.debug else "SUCCESS" if args.quiet else "INFO",
        litellm_debug=args.litellm_debug,
    )
    logger.info(f"🧠 nerve v{nerve.__version__}")

    try:
        config = Configuration.from_path(args.input_path)
    except Exception as e:
        logger.error(f"path '{args.input_path}' is not a valid agent configuration: {e}")
        raise typer.Abort() from e

    output = output or _get_output_path(args)
    cases = Cases(args.input_path)

    # apply limits from the config if available
    if config.limits:
        if config.limits.runs:
            runs = config.limits.runs
        if config.limits.max_steps:
            args.max_steps = config.limits.max_steps
        if config.limits.max_cost:
            args.max_cost = config.limits.max_cost
        if config.limits.timeout:
            args.timeout = config.limits.timeout

    eval_name = colored(args.input_path.name, "green", attrs=["bold"])
    logger.info(f"📊 {args.generator} / {eval_name} / cases: {len(cases)} / runs: {runs}")

    if output.exists():
        logger.info(f"📊 loading evaluation results from {output}")
        evaluation = Evaluation.load_from(output)
    else:
        logger.info(f"📊 saving evaluation results to {output}")
        evaluation = Evaluation.build(args, runs, len(cases))

    for case in cases:
        for run in range(runs):
            do_run = True
            if evaluation.num_runs(case.name) >= runs:
                # we already have enough runs for this case
                do_run = False
                if not evaluation.is_run_done(case.name, run):
                    # we don't have enough runs for this case
                    do_run = True
                    logger.warning(f"run {run} for {case.name} has not been completed, re-running")
                    evaluation.remove_run(case.name, run)

            if not do_run:
                logger.debug(f"skipping {case.name} ({run + 1}/{runs})")
                run_output = evaluation.get_run(case.name, run)
            else:
                logger.debug(f"running {case.name} ({run + 1}/{runs})")
                run_output = asyncio.run(_run_case(args, case))
                evaluation.add_run(case.name, run_output)

            _show_run(args, run_output, runs, run, case.name, do_run)

            if evaluation.needs_flush():
                # save at each run so we can restore later
                evaluation.save_to(output)

    logger.debug(f"evaluation results: {evaluation}")

    # save if needed
    if evaluation.needs_flush():
        evaluation.save_to(output)
        logger.info(f"📊 evaluation results saved to {output}")

    _show_results(evaluation)


def _show_run(args: Arguments, output: Output, runs: int, run: int, case_name: str, live: bool) -> None:
    usage = output.usage
    one_of = f"[{run + 1}/{runs}]" if live else f"({run + 1}/{runs})"
    subject = f"{one_of} {args.generator} / {args.input_path.name} / {case_name}"
    stats = (
        f"{output.steps} steps, {output.time:.1f} s, {usage.get('total_tokens', 0)} tokens, {usage.get('cost', 0.0)} $"
    )
    if output.task_success:
        logger.success(f"   {subject} : {stats}")
    else:
        logger.error(f"     {subject} : {stats}")


def _show_results(eval: Evaluation) -> None:
    print()
    logger.info("📊 Results")
    logger.info(f"Model: {eval.args['generator']}")
    logger.info(f"Cases: {eval.stats.cases}")
    logger.info(f"Runs: {eval.stats.runs}")
    logger.info(f"Pass: {eval.stats.passed}")
    logger.info(f"Fail: {eval.stats.failed}")

    total_cost = 0.0
    # total_tokens = 0
    total_steps = 0
    total_time = 0.0
    total_tests = eval.stats.passed + eval.stats.failed
    score = eval.stats.passed / total_tests * 100

    for _case_name, case_runs in eval.runs.items():
        for run in case_runs:
            total_cost += run.usage.get("cost", 0.0)
            # total_tokens += run.usage.get("total_tokens", 0)
            total_steps += run.steps
            total_time += run.time

    logger.info(f"Total cost: {total_cost:.2f} $")
    logger.info(f"Total time: {total_time:.2f} s")
    logger.info(f"Avg time: {total_time / total_tests:.2f} s")
    logger.info(f"Avg steps: {total_steps / total_tests:.2f}")
    logger.info(f"Avg cost: {total_cost / total_tests} $")

    logger.info(f"Score: {score:.2f} %")


async def _run_case(args: Arguments, case: Case) -> Output:
    return await Runner(
        args,
        case.input_state,
    ).run()
