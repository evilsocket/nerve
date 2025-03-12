"""
Let the agent autonomously set the task as complete or failed.
"""
import typing as t

import nerve.runtime.state as state

# for docs
EMOJI = "✅"


def task_complete_success(
    reason: t.Annotated[
        str | None, "Optional reason why the task is complete or report of conclusive information."
    ] = None,
) -> None:
    """When your objective has been reached use this tool to set the task as complete."""

    state.set_task_complete(reason)


def task_failed(
    reason: t.Annotated[str, "The reason why the task is impossible"],
) -> None:
    """Use this tool if you determine that the given goal or task is impossible given the information you have."""

    state.set_task_failed(reason)
