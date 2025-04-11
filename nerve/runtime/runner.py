import asyncio
import atexit
import json
import os
import pathlib
import sys
import time
import typing as t
import uuid

from loguru import logger
from pydantic import BaseModel

from nerve.defaults import DEFAULT_RUNS_PATH


class Arguments(BaseModel):
    input_path: pathlib.Path
    task: str | None
    generator: str
    conversation_strategy_string: str
    conversation_strategy: t.Any
    interactive: bool
    debug: bool
    litellm_debug: bool
    litellm_tracing: str | None
    quiet: bool
    max_steps: int
    max_cost: float
    timeout: int | None
    log_path: pathlib.Path | None
    trace: pathlib.Path | None
    start_state: dict[str, t.Any]

    def to_serializable(self) -> dict[str, t.Any]:
        return {
            k: v if not isinstance(v, pathlib.Path) else v.as_posix()
            for k, v in self.model_dump().items()
            if k != "conversation_strategy"
        }


def _create_command_line(
    run_args: Arguments,
    input_state: dict[str, str],
    events_file: pathlib.Path,
) -> list[str]:
    nerve_bin = sys.argv[0]
    command_line = [
        nerve_bin,
        "run",
        str(run_args.input_path),
        "--generator",
        run_args.generator,
        "--conversation",
        run_args.conversation_strategy_string,
        "--max-steps",
        str(run_args.max_steps),
        "--max-cost",
        str(run_args.max_cost),
    ]

    if run_args.timeout:
        command_line.append("--timeout")
        command_line.append(str(run_args.timeout))

    # if run_args.quiet:
    #     command_line.append("--quiet")

    # if run_args.debug:
    #     command_line.append("--debug")

    # if run_args.litellm_debug:
    #     command_line.append("--litellm-debug")

    if run_args.litellm_tracing:
        command_line.append("--litellm-tracing")
        command_line.append(run_args.litellm_tracing)

    # if the task is set, add it to the command line
    if "task" in input_state:
        command_line.append("--task")
        command_line.append(input_state["task"])
        del input_state["task"]

    command_line.append("--start-state")
    command_line.append(json.dumps(input_state))

    command_line.append("--trace")
    command_line.append(str(events_file))

    logger.debug(f"command_line: {command_line}")
    return command_line


def _get_last_event_with_name(events: list[dict[str, t.Any]], name: str) -> dict[str, t.Any] | None:
    for event in reversed(events):
        if event["name"] == name:
            return event
    return None


class ParsedEvents(BaseModel):
    output_object: dict[str, t.Any] | None
    task_success: bool
    steps: int
    time: float
    usage: dict[str, t.Any]


def _parse_events(inputs: dict[str, str], events: list[dict[str, t.Any]]) -> ParsedEvents:
    started_at = events[0].get("timestamp", 0.0)
    # we'll need this either case
    flow_completed = _get_last_event_with_name(events, "flow_complete")
    # prepare the output object
    parsed = ParsedEvents(output_object=None, task_success=False, steps=0, time=0, usage={})
    # set what we can for now
    if flow_completed is not None:
        parsed.steps = flow_completed.get("data", {}).get("steps", 0)
        parsed.usage = flow_completed.get("data", {}).get("usage", {})
        parsed.time = flow_completed.get("timestamp", 0.0) - started_at
        logger.debug(f"flow_completed: {parsed.steps} steps, {parsed.time}s, {parsed.usage}")

    # one of the tools wrote an output variable and set the task to complete
    task_completed = _get_last_event_with_name(events, "task_complete")
    if task_completed is not None:
        data = task_completed.get("data", {})
        reason = data.get("reason", {})
        if reason:
            parsed.output_object = {"reason": reason}
        else:
            parsed.output_object = data

        parsed.task_success = True
        return parsed

    # task failed
    task_failed = _get_last_event_with_name(events, "task_failed")
    if task_failed is not None:
        data = task_failed.get("data", {})
        reason = data.get("reason", {})
        if reason:
            parsed.output_object = {"reason": reason}
        else:
            parsed.output_object = data

        parsed.task_success = False
        return parsed

    # the flow completed successfully and a variable has been written (by the tool
    # that completed the task) to the output state. this is in theory redundant, but
    # we keep it for now to be safe
    if flow_completed:
        variables = flow_completed.get("data", {}).get("state", {}).get("variables", {})
        parsed.output_object = {name: value for name, value in variables.items() if name not in inputs}
        return parsed

    # fallback to the latest tool call output or text response
    # whatever comes first
    for event in reversed(events):
        if event["name"] == "text_response":
            parsed.output_object = {"response": event["data"]["response"]}
            return parsed

        elif event["name"] == "tool_called":
            parsed.output_object = {"output": event["data"]["result"]}
            return parsed

    return parsed


async def _default_stdout_fn(x: str) -> None:
    logger.debug(x)


async def _default_stderr_fn(x: str) -> None:
    logger.debug(x)


class Output(BaseModel):
    generated_at: float
    command_line: list[str]
    exit_code: int
    stdout: list[str]
    stderr: list[str]
    events: list[dict[str, t.Any]]
    output: dict[str, t.Any]
    task_success: bool
    steps: int
    time: float
    usage: dict[str, t.Any]


class Runner:
    def __init__(
        self,
        args: Arguments,
        input_state: dict[str, str] | None = None,
        id: str | None = None,
        base_path: pathlib.Path = DEFAULT_RUNS_PATH,
        clean_at_exit: bool = True,
    ):
        if not base_path.exists():
            base_path.mkdir(parents=True, exist_ok=True)

        self.id = id or str(uuid.uuid4())
        self.events_file = base_path / f"run-{self.id}.jsonl"
        self.input_state = input_state or {}
        self.command_line = _create_command_line(
            args,
            self.input_state,
            self.events_file,
        )
        self._stdout_fn: t.Callable[[str], t.Awaitable[None]] = _default_stdout_fn
        self._stderr_fn: t.Callable[[str], t.Awaitable[None]] = _default_stderr_fn
        self._process: asyncio.subprocess.Process | None = None

        if clean_at_exit:
            atexit.register(self._clean_up)

    def _clean_up(self) -> None:
        if self.events_file.exists():
            logger.debug(f"removing events file {self.events_file}")
            if self._process is not None:
                self._process.kill()
                self._process = None
            self.events_file.unlink()

    def set_stdout_fn(self, fn: t.Callable[[str], t.Awaitable[None]]) -> None:
        self._stdout_fn = fn

    def set_stderr_fn(self, fn: t.Callable[[str], t.Awaitable[None]]) -> None:
        self._stderr_fn = fn

    async def run(self) -> Output:
        logger.debug(f"spawning runner {self.id} for inputs: {self.input_state}")

        generated_at = time.time()
        outerr: dict[str, list[str]] = {
            "stdout": [],
            "stderr": [],
        }

        async def read_stream(stream: asyncio.StreamReader | None, name: str) -> None:
            nonlocal outerr

            while True and stream is not None:
                line = await stream.readline()
                if not line:
                    break

                if name == "stdout":
                    await self._stdout_fn(line.decode().rstrip())
                else:
                    await self._stderr_fn(line.decode().rstrip())

                outerr[name].append(line.decode().rstrip())

        self._process = await asyncio.create_subprocess_exec(
            *self.command_line,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            env=os.environ.copy(),
        )
        stdout_task = asyncio.create_task(read_stream(self._process.stdout, "stdout"))
        stderr_task = asyncio.create_task(read_stream(self._process.stderr, "stderr"))

        # wait for the process and stdout/stderr readers to complete
        await self._process.wait()
        await asyncio.gather(stdout_task, stderr_task)

        logger.debug(f"process exited with code {self._process.returncode}, reading events ...")

        # read the events file
        events = []
        with open(self.events_file) as f:
            for line in f:
                events.append(json.loads(line))

        logger.debug(f"read {len(events)} events")

        events.sort(key=lambda event: event.get("timestamp", 0), reverse=False)

        parsed = _parse_events(self.input_state, events)
        if parsed.output_object is None:
            logger.warning(f"could not get raw output value from runner {self.id}")

            if outerr["stderr"]:
                parsed.output_object = {"output": "\n".join(outerr["stderr"])}
            elif outerr["stdout"]:
                parsed.output_object = {"output": "\n".join(outerr["stdout"])}
            else:
                parsed.output_object = {"output": "the tool did not write any output"}

        logger.debug(f"output value: {parsed.output_object}")

        exit_code = self._process.returncode or 0
        self._process = None

        return Output(
            generated_at=generated_at,
            command_line=self.command_line,
            exit_code=exit_code,
            stdout=outerr["stdout"],
            stderr=outerr["stderr"],
            events=events,
            output=parsed.output_object,
            task_success=parsed.task_success,
            steps=parsed.steps,
            time=parsed.time,
            usage=parsed.usage,
        )
