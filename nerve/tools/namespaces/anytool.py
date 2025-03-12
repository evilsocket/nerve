"""
Let the agent create its own tools in Python.
"""

import typing as t
from typing import Annotated

from loguru import logger
from pydantic import Field

from nerve.runtime import state
from nerve.tools.compiler import wrap_tool_function

# for docs
EMOJI = "🔧"


def create_tool(
    code: Annotated[
        str,
        Field(
            description="The Python code to create the tool.",
            examples=[
                '''
# import any standard library you need
from typing import Annotated

def this_will_be_the_tool_name(
    argument_1: Annotated[str, "The first argument"],
    argument_2: Annotated[str, "The second argument"],
) -> str:
    """ALWAYS add a docstring to the tool."""
    return "This is the return value of the tool."
'''
            ],
        ),
    ],
) -> None:
    """Create a new tool or redefine an existing one by defining it as an annotated Python function.
    Use this tool to implement the missing functionalities you need to perform your task."""

    func_namespace: dict[str, t.Any] = {}
    exec(code, func_namespace)

    for name, value in func_namespace.items():
        if name[0] == "_":
            continue

        elif not callable(value):
            continue

        elif value.__module__ is not None:
            continue

        logger.debug(f"creating tool: {name}")
        tool_fn = wrap_tool_function(value)
        state.set_extra_tool(tool_fn)
