import pathlib
import sys
import typing as t

from loguru import logger

from nerve.runtime import state
from nerve.runtime.events import Event


def init(log_path: pathlib.Path | None = None, debug: bool = False, litellm_debug: bool = False) -> None:
    """
    Initialize the logging system.

    Args:
        log_path: The path to the log file.
        debug: Whether to enable debug logging.
        litellm_debug: Whether to enable litellm debug logging.
    """
    level = "DEBUG" if debug else "INFO"
    format = "<green>{time}</green> <level>{message}</level>"

    if not debug:
        logger.remove()
        logger.add(
            sys.stdout,
            colorize=True,
            format=format,
            level=level,
        )

    if litellm_debug:
        import litellm

        litellm._turn_on_debug()  # type: ignore

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
        timeout = data["flow"].timeout
        timeout = f"{timeout}s timeout" if timeout else "no timeout"
        conv_window_strategy = data["flow"].actors[0].conv_window_strategy
        logger.info(f"🚀 {max_steps} max steps | {timeout} | {conv_window_strategy}")

    elif event.name == "agent_created":
        if isinstance(data["agent"], dict):
            data["agent"] = DictWrapper(data["agent"])

        generator = data["agent"].runtime.generator
        name = data["agent"].runtime.name
        version = data["agent"].configuration.version
        tools = len(data["agent"].runtime.tools)
        logger.info(f"🤖 {generator} | {name} v{version} with {tools} tools")

    elif event.name == "before_tool_called":
        args_str = ", ".join([f"{k}={v}" for k, v in data["args"].items()])
        logger.debug(f"🛠️  {data['name']}({args_str}) ...")

    elif event.name == "tool_called":
        # avoid logging twice
        if data["name"] in ("task_complete_success"):
            return

        elapsed_time = data["finished_at"] - data["started_at"]
        args_str = ", ".join([f"{k}={v}" for k, v in data["args"].items()])

        if data["result"] is None:
            ret = ""
        else:
            ret = f"{len(str(data['result']))} bytes in "

        logger.info(f"🛠️  {data['name']}({args_str}) -> {ret}{elapsed_time:.4f} seconds")

    elif event.name == "task_complete":
        if isinstance(data["actor"], dict):
            data["actor"] = DictWrapper(data["actor"])

        reason = f": {data['reason']}" if data["reason"] else ""
        logger.info(f"✅ task {data['actor'].runtime.name} completed{reason}")

    elif event.name == "task_failed":
        logger.error(f"❌ task {data['actor'].runtime.name} failed: {data['reason']}")

    elif event.name == "tool_created":
        logger.info(f"🧰 registered tool: {data['name']}")

    elif event.name == "unknown_tool":
        logger.warning(f"❌ model called unknown tool: {data['tool_name']}")

    elif event.name == "tool_error":
        logger.error(f"❌ error executing {data['tool_name']}({data['args']}): {data['error']}")

    elif event.name == "flow_complete":
        logger.info(f"⚙️  flow complete in {data['steps']} steps")

    elif event.name == "text_response":
        logger.info(f"💬 {data['response']}")

    elif event.name == "step_started":
        if isinstance(data["token_usage"], dict):
            data["token_usage"] = DictWrapper(data["token_usage"])

        if data["token_usage"].total_tokens > 0:
            logger.info(f"📊 [step {data['step']}] [usage {data['token_usage']}]")
        else:
            logger.info(f"📊 [step {data['step']}]")

    elif event.name in ("task_started", "agent_step", "step_complete", "variable_change", "knowledge_change"):
        pass
    else:
        logger.info(f"unknown event: {event}")
