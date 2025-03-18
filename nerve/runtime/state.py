import asyncio
import json
import os
import pathlib
import threading
import typing as t

import click
import jinja2
from loguru import logger

from nerve.models import Mode, Status, Usage
from nerve.runtime.events import Event
from nerve.runtime.thread_pool import ThreadPool

# the current actor
_current_actor: t.Any | None = None
# total usage
_usage: Usage = Usage()
# event log
_events: list[Event] = []
# trace file
_trace_file: pathlib.Path | None = None
# working mode
_mode: Mode = Mode.AUTOMATIC
# the status of the active task
_task_status: Status = Status.RUNNING
# the reason for failed status
_reason: str | None = None
# variables
_variables: dict[str, t.Any] = {}
# defaults for variables
_defaults: dict[str, t.Any] = {}
# similar to variables but used by tools
_knowledge: dict[str, t.Any] = {}
# tools
_tools: dict[str, t.Callable[..., t.Any]] = {}
# extra tools defined at runtime
_extra_tools: dict[str, t.Callable[..., t.Any]] = {}
# listeners for events
_listeners: list[t.Callable[[Event], None]] = []
# thread pool for asynchronous event dispatching
_thread_pool: ThreadPool = ThreadPool()


def add_event_listener(listener: t.Callable[[Event], None]) -> None:
    """Add a listener function for events."""

    global _listeners
    _listeners.append(listener)


def set_trace_file(trace_file: pathlib.Path) -> None:
    """Enable recording of events to a file."""

    global _trace_file

    if trace_file.exists():
        logger.error(f"trace file {trace_file} already exists")
        exit(1)

    _trace_file = trace_file.absolute()
    logger.info(f"🔍 tracing to {_trace_file}")


class CustomJSONEncoder(json.JSONEncoder):
    def default(self, o: t.Any) -> t.Any:
        logger.debug("serializing", o)
        if hasattr(o, "model_dump"):
            return o.model_dump()
        elif hasattr(o, "__dict__"):
            return o.__dict__
        elif hasattr(o, "__str__"):
            return str(o)

        return super().default(o)


def wait_for_events_logs() -> None:
    """Wait for all events to be flushed."""

    global _thread_pool
    _thread_pool.wait_all()


def update_usage(usage: Usage) -> None:
    """Update the total usage."""

    global _usage
    _usage += usage


def get_usage() -> Usage:
    """Get the total usage."""

    global _usage
    return _usage


def on_event(name: str, data: t.Any | None = None) -> None:
    """Register an event."""

    global _events

    event = Event(name=name, data=data)
    _events.append(event)

    for listener in _listeners:
        # execute the listener in a separate thread in order to avoid blocking the main thread
        _task_id = _thread_pool.submit(listener, event)

    if _trace_file:
        with open(_trace_file, "a+t") as f:
            line = json.dumps(event.model_dump(), cls=CustomJSONEncoder)
            f.write(f"{line}\n")


def on_task_started(actor: t.Any) -> None:
    """Register a task start."""

    global _current_actor
    _current_actor = actor
    on_event("task_started", {"actor": actor})


def get_current_actor() -> t.Any:
    """Get the current actor."""

    global _current_actor
    return _current_actor


def on_before_tool_called(
    name: str,
    args: t.Any | None = None,
) -> None:
    """Register a tool call (before it is executed)."""

    on_event(
        "before_tool_called",
        {
            "name": name,
            "args": args,
        },
    )


def on_tool_called(
    started_at: float,
    finished_at: float,
    name: str,
    args: t.Any | None = None,
    result: t.Any | None = None,
    error: t.Any | None = None,
) -> None:
    """Register a tool call (after it is executed)."""

    on_event(
        "tool_called",
        {
            "started_at": started_at,
            "finished_at": finished_at,
            "name": name,
            "args": args,
            "result": result,
            "error": error,
        },
    )


def set_mode(new_mode: Mode) -> None:
    """Set the mode."""

    global _mode

    if _mode != new_mode:
        on_event("mode_change", {"from": _mode, "to": new_mode})

    _mode = new_mode


def get_mode() -> Mode:
    """Get the current mode."""

    global _mode
    return _mode


def is_interactive() -> bool:
    """Check if the current mode is interactive."""

    return get_mode() == Mode.INTERACTIVE


def set_defaults(defaults: dict[str, t.Any]) -> None:
    """Set the defaults for variables."""

    global _defaults
    _defaults = defaults


def is_active_task_done() -> bool:
    """Check if the active task is done."""

    return _task_status.is_done()


def set_task_complete(the_reason: str | None = None) -> None:
    """Set the task as complete."""

    global _task_status, _reason
    _task_status = Status.COMPLETED
    _reason = the_reason
    on_event(
        "task_complete", {"actor": _current_actor.runtime.name if _current_actor else "unknown", "reason": the_reason}
    )


def set_task_failed(the_reason: str) -> None:
    """Set the task as failed."""

    global _task_status, _reason
    _task_status = Status.FAILED
    _reason = the_reason

    on_event(
        "task_failed", {"actor": _current_actor.runtime.name if _current_actor else "unknown", "reason": the_reason}
    )


def on_max_steps_reached() -> None:
    """Set the task as failed due to max steps reached."""

    global _task_status, _reason
    if _task_status == Status.RUNNING:
        set_task_failed("max steps reached")


def on_max_cost_reached() -> None:
    """Set the task as failed due to max cost reached."""

    global _task_status, _reason
    if _task_status == Status.RUNNING:
        set_task_failed("max cost reached")


def on_timeout() -> None:
    """Set the task as failed due to timeout."""

    global _task_status, _reason
    if _task_status == Status.RUNNING:
        set_task_failed("timeout reached")


def as_dict() -> dict[str, t.Any]:
    """Get the current state as a dictionary."""

    return {
        "mode": _mode,
        "current_task": {
            "status": _task_status,
            "reason": _reason,
        },
        "variables": _variables,
        "knowledge": _knowledge,
    }


def set_tools(tools: dict[str, t.Callable[..., t.Any]]) -> None:
    """Set all tools."""

    global _tools
    _tools = tools

    logger.debug(f"tools: {_tools}")


def get_tools() -> dict[str, t.Callable[..., t.Any]]:
    """Get all tools."""

    global _tools
    return _tools


def get_extra_tools() -> dict[str, t.Callable[..., t.Any]]:
    """Get any extra tool registered at runtime."""

    global _extra_tools
    return _extra_tools


def set_extra_tool(tool: t.Callable[..., t.Any]) -> None:
    """Register an extra tool at runtime (used by anytool namespace)."""

    global _extra_tools
    _extra_tools[tool.__name__] = tool
    on_event("tool_created", {"name": tool.__name__, "code": tool.__code__.co_code})


def get_variable(key: str, default: t.Any = None) -> t.Any:
    """Get a variable."""

    global _variables
    return _variables.get(key, default)


def get_variables() -> dict[str, t.Any]:
    """Get all variables."""

    global _variables
    return _variables


def get_knowledge() -> dict[str, t.Any]:
    """Get the knowledge variable (a piece of information appearing in the system prompt)."""

    global _knowledge
    return _knowledge


def write_knowledge(key: str, value: t.Any) -> None:
    """Write a piece of knowledge that will be used in the system prompt."""

    global _knowledge

    on_event("knowledge_change", {"name": key, "from": _knowledge.get(key), "to": value})

    _knowledge[key] = value


def append_to_knowledge(key: str, value: t.Any) -> None:
    """Append a piece of knowledge to the existing knowledge."""

    global _knowledge
    if key not in _knowledge:
        write_knowledge(key, value)
    else:
        write_knowledge(key, _knowledge[key] + "\n" + value)


def clear_knowledge(key: str) -> None:
    """Remove a piece of knowledge."""

    global _knowledge
    if key in _knowledge:
        on_event("knowledge_change", {"name": key, "from": _knowledge[key], "to": None})
        del _knowledge[key]


def update_variables(update: dict[str, t.Any]) -> None:
    """Update variables."""

    global _variables
    for key, value in update.items():
        on_event("variable_change", {"name": key, "from": _variables.get(key), "to": value})
    _variables.update(update)


def reset() -> None:
    """Reset the state."""

    global _task_status, _reason, _knowledge
    _task_status = Status.RUNNING
    _reason = None
    _knowledge = {}


def on_user_input_needed(input_name: str, prompt: str) -> str:
    """Get user input."""

    # check if defined as environment variable
    if input_name in os.environ:
        return os.environ[input_name]

    elif input_name in _variables:
        return str(_variables[input_name])

    elif input_name in _defaults:
        # from the agent defaults
        return str(_defaults[input_name])

    elif is_interactive():
        # wait for all events to be logged
        wait_for_events_logs()
        # ask the user
        return input(prompt).strip()

    else:
        raise click.MissingParameter(
            param=click.Option(
                [f"--{input_name}"],
                type=str,
                required=True,
            ),
            message=f"Command line argument --{input_name} is required in non interactive mode.",
        )


def _create_jinja_env() -> jinja2.Environment:
    # we use this to catch undefined variables at runtime
    class OnUndefinedVariable(jinja2.Undefined):
        def __init__(self, *args: t.Any, **kwargs: t.Any) -> None:
            super().__init__(*args, **kwargs)

            global _variables

            undefined_name = self._undefined_name or ""

            logger.debug(f"undefined variable encountered: {undefined_name}")
            logger.debug(f"current variables: {_variables}")
            logger.debug(f"current defaults: {_defaults}")

            self.value = on_user_input_needed(undefined_name, f"Enter value for '{undefined_name}': ")

            # save to state
            update_variables({undefined_name or "": self.value or ""})

        def __str__(self) -> str:
            return self.value or "<UNDEFINED>"

    env = jinja2.Environment(undefined=OnUndefinedVariable, loader=jinja2.FileSystemLoader("."))

    # allow prompts to call tools
    for name, tool_fn in _tools.items():
        # if the tool is async, wrap it in a sync function to make it callable from Jinja
        if asyncio.iscoroutinefunction(tool_fn):

            def make_sync_wrapper(fn: t.Callable[..., t.Any]) -> t.Callable[..., t.Any]:
                def sync_wrapper(*args: t.Any, **kwargs: t.Any) -> t.Any:
                    logger.debug(f"calling tool {fn.__name__} from template")
                    coro = fn(*args, **kwargs)
                    # NOTE: we use a list to store the result because nonlocal variables (like scalars) inside a nested
                    # function do not allow assignment unless explicitly declared nonlocal. However, mutable objects like
                    # lists can be modified within the nested function without extra declarations.
                    result_container = []

                    def we_need_an_async_loop_thread() -> None:
                        loop = asyncio.new_event_loop()
                        asyncio.set_event_loop(loop)
                        result_container.append(loop.run_until_complete(coro))

                    thread = threading.Thread(target=we_need_an_async_loop_thread)
                    thread.start()
                    thread.join()

                    return result_container[0]

                return sync_wrapper

            env.globals[name] = make_sync_wrapper(tool_fn)
        else:
            env.globals[name] = tool_fn

    return env


def interpolate(raw: str, extra: dict[str, t.Any] | None = None) -> str:
    """Interpolate the current state into a string."""

    env = _create_jinja_env()
    template = env.from_string(raw)
    context = _variables | (extra or {})

    return template.render(**context)
