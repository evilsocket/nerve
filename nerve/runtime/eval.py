import json
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

from nerve.runtime.runner import Arguments, Output


class Statistics(BaseModel):
    generator: str = ""
    max_steps: int = 0
    max_cost: float = 0
    timeout: float = 0
    window: str = ""
    runs: int = 0
    cases: int = 0
    passed: int = 0
    failed: int = 0


class Evaluation(BaseModel):
    name: str = ""
    started_at: float = 0.0
    finished_at: float = 0.0
    args: dict[str, t.Any] = {}
    runs: dict[str, list[Output]] = {}
    stats: Statistics = Statistics()

    _flush: bool = False

    @classmethod
    def build(cls, args: Arguments, runs: int, cases: int) -> "Evaluation":
        return Evaluation(
            started_at=time.time(),
            name=args.input_path.name,
            args=args.to_serializable(),
            stats=Statistics(
                generator=args.generator,
                max_steps=args.max_steps,
                max_cost=args.max_cost,
                timeout=args.timeout or 0.0,
                window=args.conversation_strategy_string,
                runs=runs,
                cases=cases,
            ),
        )

    def add_run(self, case_name: str, run_output: Output) -> None:
        if case_name not in self.runs:
            self.runs[case_name] = []

        self.runs[case_name].append(run_output)
        if run_output.task_success:
            self.stats.passed += 1
        else:
            self.stats.failed += 1

        self._flush = True

    def remove_run(self, case_name: str, run_idx: int) -> None:
        run_output = self.runs[case_name][run_idx]
        if run_output.steps > 0:
            # we're remove a completed run, take counters into account
            if run_output.task_success:
                self.stats.passed -= 1
            else:
                self.stats.failed -= 1

        self.runs[case_name].pop(run_idx)
        self._flush = True

    def num_runs(self, case_name: str) -> int:
        return len(self.runs[case_name]) if case_name in self.runs else 0

    def num_run_steps(self, case_name: str, run_idx: int) -> int:
        return self.runs[case_name][run_idx].steps

    def is_run_done(self, case_name: str, run_idx: int) -> bool:
        return self.num_run_steps(case_name, run_idx) > 0

    def get_run(self, case_name: str, run_idx: int) -> Output:
        return self.runs[case_name][run_idx]

    def needs_flush(self) -> bool:
        return self._flush

    def save_to(self, path: pathlib.Path) -> None:
        self.finished_at = time.time()
        path.write_text(self.model_dump_json())
        self._flush = False

    @classmethod
    def load_from(cls, path: pathlib.Path) -> "Evaluation":
        data = json.loads(path.read_text())
        return cls.model_validate(data)


class Case(BaseModel):
    name: str
    input_state: dict[str, t.Any]


class Cases:
    class Source(Enum):
        # cases have their own individual folders
        FOLDER = 0
        # cases are listed in a single file
        YAML = 1
        # parquet file
        PARQUET = 2

    def __init__(self, eval_path: pathlib.Path):
        self._eval_path = eval_path
        self._cases: list[Case] = []
        self._source = self.Source.FOLDER

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

    def _from_folder(self, cases_folder: pathlib.Path) -> None:
        logger.info(f"📊 loading evaluation cases from folder {cases_folder}")
        self._source = self.Source.FOLDER
        for path in natsorted(cases_folder.glob("*")):
            self._cases.append(
                Case(
                    name=path.name,
                    input_state={
                        "CASE_NAME": path.name,
                        "CASE_PATH": path.absolute().as_posix(),
                    },
                )
            )

    def _from_yaml(self, cases_file: pathlib.Path) -> None:
        logger.info(f"📊 loading evaluation cases from file {cases_file}")
        self._source = self.Source.YAML
        for case in parse_yaml_file_as(list[dict[str, dict[str, t.Any]]], cases_file):  # type: ignore[type-var]
            for case_name, input_state in case.items():
                self._cases.append(Case(name=case_name, input_state=input_state))

    def _from_parquet(self, cases_file: pathlib.Path) -> None:
        logger.info(f"📊 loading evaluation cases from parquet file {cases_file}")
        self._source = self.Source.PARQUET
        pf = ParquetFile(cases_file)
        df = pf.to_pandas()
        num_rows = len(df)
        for index, row in df.iterrows():
            self._cases.append(
                Case(
                    name=f"case_{index}_of_{num_rows}",
                    input_state=row.to_dict(),
                )
            )

    def __iter__(self) -> t.Iterator[Case]:
        return iter(self._cases)

    def __len__(self) -> int:
        return len(self._cases)
