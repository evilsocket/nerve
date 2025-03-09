"""
Let the agent execute bash commands.
"""

import subprocess
from typing import Annotated


def execute_bash_command(
    command: Annotated[str, "The bash command to execute"],
) -> str:
    """Execute a bash command and return the output."""

    # TODO: this will execute in the system shell which may not be bash ...
    return subprocess.check_output(command, shell=True).decode("utf-8")
