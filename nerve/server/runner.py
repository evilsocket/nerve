import asyncio
import json
import os
import pathlib
import sys
import tempfile
import typing as t
import uuid

from loguru import logger


def _create_command_line(
    input_path: pathlib.Path,
    generator: str,
    conversation_strategy: str,
    max_steps: int,
    max_cost: float,
    timeout: int | None,
    quiet: bool,
    input_state: dict[str, str],
    events_file: pathlib.Path,
) -> list[str]:
    nerve_bin = sys.argv[0]
    command_line = [
        nerve_bin,
        "run",
        str(input_path),
        "--generator",
        generator,
        "--conversation",
        conversation_strategy,
        "--max-steps",
        str(max_steps),
        "--max-cost",
        str(max_cost),
    ]

    if timeout:
        command_line.append("--timeout")
        command_line.append(str(timeout))

    if quiet:
        command_line.append("--quiet")

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


def _get_output_object(inputs: dict[str, str], events: list[dict[str, t.Any]]) -> dict[str, t.Any] | None:
    # one of the tools wrote an output variable and set the task to complete
    completed = _get_last_event_with_name(events, "task_complete")
    if completed is not None:
        outputs = completed.get("data", {}).get("reason", {})
        if outputs:
            return outputs  # type: ignore

    # task failed
    failed = _get_last_event_with_name(events, "task_failed")
    if failed is not None:
        outputs = failed.get("data", {}).get("reason", {})
        if outputs:
            return outputs  # type: ignore

    # the flow completed successfully and a variable has been written (by the tool
    # that completed the task) to the output state. this is in theory redundant, but
    # we keep it for now to be safe
    completed = _get_last_event_with_name(events, "flow_complete")
    if completed is not None:
        variables = completed.get("data", {}).get("state", {}).get("variables", {})
        outputs = {name: value for name, value in variables.items() if name not in inputs}
        if outputs:
            return outputs  # type: ignore

    # fallback to the latest tool call output or text response
    # whatever comes first
    for event in reversed(events):
        if event["name"] == "text_response":
            return {"response": event["data"]["response"]}

        elif event["name"] == "tool_called":
            return {"output": event["data"]["result"]}

    return None


class Runner:
    def __init__(
        self,
        input_path: pathlib.Path,
        generator: str,
        conversation_strategy: str,
        max_steps: int,
        max_cost: float,
        timeout: int | None,
        quiet: bool,
        input_state: dict[str, str],
    ):
        self.id = uuid.uuid4()
        self.events_file = pathlib.Path(tempfile.gettempdir()) / f"nerve-runner-{self.id}.jsonl"
        self.input_state = input_state
        self.command_line = _create_command_line(
            input_path,
            generator,
            conversation_strategy,
            max_steps,
            max_cost,
            timeout,
            quiet,
            input_state,
            self.events_file,
        )
        self._stdout_fn: t.Callable[[str], t.Awaitable[None]] | None = None
        self._stderr_fn: t.Callable[[str], t.Awaitable[None]] | None = None

    def set_stdout_fn(self, fn: t.Callable[[str], t.Awaitable[None]]) -> None:
        self._stdout_fn = fn

    def set_stderr_fn(self, fn: t.Callable[[str], t.Awaitable[None]]) -> None:
        self._stderr_fn = fn

    async def run(self) -> dict[str, t.Any]:
        logger.info(f"spawning runner {self.id} for inputs: {self.input_state}")

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
                    if self._stdout_fn:
                        await self._stdout_fn(line.decode().rstrip())
                else:
                    if self._stderr_fn:
                        await self._stderr_fn(line.decode().rstrip())

                outerr[name].append(line.decode().rstrip())

        process = await asyncio.create_subprocess_exec(
            *self.command_line,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            env=os.environ.copy(),
        )
        stdout_task = asyncio.create_task(read_stream(process.stdout, "stdout"))
        stderr_task = asyncio.create_task(read_stream(process.stderr, "stderr"))

        # wait for the process and stdout/stderr readers to complete
        await process.wait()
        await asyncio.gather(stdout_task, stderr_task)

        logger.debug(f"process exited with code {process.returncode}, reading events ...")

        # read the events file
        events = []
        with open(self.events_file) as f:
            for line in f:
                events.append(json.loads(line))

        logger.debug(f"read {len(events)} events")

        output_object = _get_output_object(self.input_state, events)
        if output_object is None:
            logger.warning(f"could not get raw output value from runner {self.id}")

            if outerr["stderr"]:
                output_object = {"output": "\n".join(outerr["stderr"])}
            elif outerr["stdout"]:
                output_object = {"output": "\n".join(outerr["stdout"])}
            else:
                output_object = {"output": "the tool did not write any output"}

        logger.debug(f"output value: {output_object}")

        output_state = {
            "command_line": self.command_line,
            "output": output_object,
            "exit_code": process.returncode,
            "stdout": outerr["stdout"],
            "stderr": outerr["stderr"],
            "events": events,
        }

        return output_state
