import asyncio
import pathlib
import time
import typing as t
from enum import Enum

import typer
from fastparquet import ParquetFile  # type: ignore[import-untyped]
from loguru import logger
from natsort import natsorted
from pydantic import BaseModel
from pydantic_yaml import parse_yaml_file_as
from termcolor import colored
from typer_di import Depends, TyperDI

import nerve
from nerve.cli.defaults import DEFAULT_EVAL_RUNS
from nerve.cli.utils import _get_run_args
from nerve.models import Configuration, Evaluation
from nerve.runtime import logging
from nerve.server.runner import Arguments, Output, Runner

cli = TyperDI(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


class CaseIterator:
    class Mode(Enum):
        # cases have their own individual folders
        FOLDER = 0
        # cases are listed in a single file
        YAML = 1
        # parquet file
        PARQUET = 2

    class Case(BaseModel):
        name: str
        input_state: dict[str, t.Any]

    def _from_folder(self, cases_folder: pathlib.Path) -> None:
        logger.info(f"📊 loading evaluation cases from folder {cases_folder}")
        self._mode = self.Mode.FOLDER
        for path in natsorted(cases_folder.glob("*")):
            self._cases.append(
                CaseIterator.Case(
                    name=path.name,
                    input_state={
                        "CASE_NAME": path.name,
                        "CASE_PATH": path.absolute().as_posix(),
                    },
                )
            )

    def _from_yaml(self, cases_file: pathlib.Path) -> None:
        logger.info(f"📊 loading evaluation cases from file {cases_file}")
        self._mode = self.Mode.YAML
        for case in parse_yaml_file_as(list[dict[str, dict[str, t.Any]]], cases_file):  # type: ignore[type-var]
            for case_name, input_state in case.items():
                self._cases.append(CaseIterator.Case(name=case_name, input_state=input_state))

    def _from_parquet(self, cases_file: pathlib.Path) -> None:
        logger.info(f"📊 loading evaluation cases from parquet file {cases_file}")
        self._mode = self.Mode.PARQUET
        pf = ParquetFile(cases_file)
        df = pf.to_pandas()
        num_rows = len(df)
        for index, row in df.iterrows():
            self._cases.append(
                CaseIterator.Case(
                    name=f"case_{index}_of_{num_rows}",
                    input_state=row.to_dict(),
                )
            )

    def __init__(self, eval_path: pathlib.Path):
        self._eval_path = eval_path
        self._cases: list[CaseIterator.Case] = []
        self._mode = self.Mode.FOLDER

        cases_folder = self._eval_path / "cases"
        cases_file_yml = self._eval_path / "cases.yml"
        cases_file_parquet = self._eval_path / "cases.parquet"

        if cases_folder.exists():
            self._from_folder(cases_folder)

        elif cases_file_yml.exists():
            self._from_yaml(cases_file_yml)

        elif cases_file_parquet.exists():
            self._from_parquet(cases_file_parquet)

        if not self._cases:
            logger.error(f"no cases found in {self._eval_path}")
            raise typer.Abort()

    def __iter__(self) -> t.Iterator["CaseIterator.Case"]:
        return iter(self._cases)

    def __len__(self) -> int:
        return len(self._cases)


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
    cases = CaseIterator(args.input_path)
    new_runs = False

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
    logger.info(f"📊 {args.generator} vs {eval_name} | cases: {len(cases)} | runs: {runs}")

    if output.exists():
        logger.info(f"📊 loading evaluation results from {output}")
        eval_result = Evaluation.load_from(output)
    else:
        logger.info(f"📊 saving evaluation results to {output}")
        eval_result = Evaluation.build(args, runs, len(cases))

    for case in cases:
        if case.name not in eval_result.cases:
            eval_result.cases[case.name] = Evaluation.Case(started_at=time.time())
            new_runs = True

        for run in range(runs):
            num_runs_done = len(eval_result.cases[case.name].runs)
            do_run = num_runs_done < (run + 1)
            if not do_run:
                # check that the run has been completed
                if eval_result.cases[case.name].runs[run].steps == 0:
                    do_run = True
                    logger.warning(f"run {run} for {case.name} has not been completed, re-running")

            logger.debug(f"got {num_runs_done} runs for {case.name}")

            if not do_run:
                logger.debug(f"skipping {case.name} ({run + 1}/{runs})")
                run_output = eval_result.cases[case.name].runs[run]
            else:
                logger.debug(f"running {case.name} ({run + 1}/{runs})")
                run_output = asyncio.run(_run_case(args, case))
                eval_result.add_run(case.name, run_output)
                new_runs = True

            usage = run_output.usage
            if run_output.task_success:
                logger.success(
                    f"   [{run + 1}/{runs}] {eval_name} / {case.name} : {run_output.steps} steps | {run_output.time:.1f} s | {usage.get('total_tokens', 0)} tokens | {usage.get('cost', 0.0)} $"
                )
            else:
                logger.error(
                    f"     [{run + 1}/{runs}] {eval_name} / {case.name} : {run_output.steps} steps | {run_output.time:.1f} s | {usage.get('total_tokens', 0)} tokens | {usage.get('cost', 0.0)} $"
                )

            if do_run:
                # save at each run so we can restore later
                eval_result.save_to(output)

    logger.debug(f"evaluation results: {eval_result}")

    # save if we did any runs
    if new_runs:
        eval_result.save_to(output)
        logger.info(f"📊 evaluation results saved to {output}")

    _show_results(eval_result)


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

    for _case_name, case in eval.cases.items():
        for run in case.runs:
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


async def _run_case(args: Arguments, case: CaseIterator.Case) -> Output:
    return await Runner(
        args,
        case.input_state,
    ).run()
