import pathlib
import sys
import typing as t

import litellm
from loguru import logger
from termcolor import colored

from nerve.runtime import state
from nerve.runtime.events import Event


def init(
    log_path: pathlib.Path | None = None,
    level: str = "INFO",
    litellm_debug: bool = False,
    litellm_tracing: str | None = None,
    target: t.TextIO = sys.stderr,
) -> None:
    """
    Initialize the logging system.

    Args:
        log_path: The path to the log file.
        level: The log level to use.
        litellm_debug: Whether to enable litellm debug logging.
        litellm_tracing: The callback to set for litellm tracing.
    """

    if level != "DEBUG":
        logger.remove()
        logger.add(
            target,
            colorize=True,
            format="<level>[{time:MM-DD-YY HH:mm:ss}] {level}</level> {message}",
            level=level,
        )

    if litellm_debug:
        litellm._turn_on_debug()  # type: ignore
    else:
        # https://github.com/BerriAI/litellm/issues/4825
        litellm.suppress_debug_info = True

    if litellm_tracing:
        litellm.callbacks = [litellm_tracing]  # type: ignore

    if log_path:
        logger.add(log_path, format=format, level=level)

    state.add_event_listener(log_event_to_terminal)


# a dynamic object that will access dictionary attributes
class DictWrapper:
    def __init__(self, data_dict: dict[str, t.Any]) -> None:
        self._data = data_dict

    def __getattr__(self, name: str) -> t.Any:
        if name in self._data:
            if isinstance(self._data[name], dict):
                return DictWrapper(self._data[name])

            elif isinstance(self._data[name], list):
                return [DictWrapper(item) for item in self._data[name]]

            return self._data[name]

        return "unknown"

    def __str__(self) -> str:
        return str(self._data)


def log_event_to_terminal(event: Event) -> None:
    data = event.data or {}
    if event.name == "flow_started":
        if isinstance(data["flow"], dict):
            data["flow"] = DictWrapper(data["flow"])

        max_steps = data["flow"].max_steps
        steps = f"max steps: {max_steps}" if max_steps > 0 else "unlimited steps"

        max_cost = data["flow"].max_cost
        cost = f"max cost: {max_cost:.2f}$" if max_cost > 0 else "no cost limit"

        timeout = data["flow"].timeout
        timeout = f"{timeout}s timeout" if timeout else "no timeout"

        conv_window_strategy = data["flow"].actors[0].conv_window_strategy
        logger.info(f"🚀 {steps} | {cost} | {timeout} | {conv_window_strategy}")

    elif event.name == "agent_created":
        if isinstance(data["agent"], dict):
            data["agent"] = DictWrapper(data["agent"])

        generator = data["agent"].runtime.generator
        name = data["agent"].runtime.name
        version = data["agent"].configuration.version
        tools = len(data["agent"].runtime.tools)
        logger.info(f"🤖 {generator} | {name} v{version} with {tools} tools")

    elif event.name == "before_tool_called":
        # avoid logging twice
        if data["name"] in ("task_complete", "task_failed"):
            return

        args_str = ", ".join([colored(v, "yellow") for v in data["args"].values()])
        name = colored(data["name"], attrs=["bold"])
        logger.info(f"🛠️  {name}({args_str})")

    elif event.name == "tool_called":
        # avoid logging twice
        if data["name"] in ("task_complete", "task_failed"):
            return

        elapsed_time = data["finished_at"] - data["started_at"]
        if data["result"] is None:
            ret = "<none>"
        else:
            ret = f"{type(data['result'])} ({len(str(data['result']))} bytes)"

        logger.info(colored(f" ↳ {data['name']} -> {ret} in {elapsed_time:.4f}s", "dark_grey"))

    elif event.name == "task_complete":
        logger.info(colored(f"✅ task {data['actor']} completed", "green", attrs=["bold"]))

    elif event.name == "task_failed":
        logger.error(colored(f"❌ task {data['actor']} failed: {data['reason']}", "red", attrs=["bold"]))

    elif event.name == "tool_created":
        logger.info(f"🧰 registered tool: {data['name']}")

    elif event.name == "unknown_tool":
        logger.warning(f"❌ model called unknown tool: {data['tool_name']}")

    elif event.name == "tool_error":
        logger.error(f"❌ error executing {data['tool_name']}({data['args']}): {data['error']}")

    elif event.name == "flow_complete":
        if isinstance(data["usage"], dict):
            data["usage"] = DictWrapper(data["usage"])

        if data["usage"].total_tokens > 0:
            logger.info(f"⚙️  flow complete in {data['steps']} steps ({data['usage']})")
        else:
            logger.info(f"⚙️  flow complete in {data['steps']} steps")

    elif event.name == "text_response":
        logger.info(f"💬 {colored(data['response'], 'black', 'on_white')}")

    elif event.name == "step_started":
        if isinstance(data["usage"], dict):
            data["usage"] = DictWrapper(data["usage"])

        if data["usage"].total_tokens > 0:
            logger.info(f"📊 [step {data['step']}] [usage {data['usage']}]")
        else:
            logger.info(f"📊 [step {data['step']}]")

    elif event.name in (
        "task_started",
        "agent_step",
        "step_complete",
        "variable_change",
        "knowledge_change",
        "mode_change",
    ):
        pass
    else:
        logger.info(f"unknown event: {event}")
