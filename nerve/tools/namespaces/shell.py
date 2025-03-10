"""
Let the agent execute shell commands.
"""

import subprocess
from typing import Annotated


def execute_shell_command(
    command: Annotated[str, "The shell command to execute"],
) -> str:
    """Execute a shell command and return the output."""

    return subprocess.check_output(command, shell=True).decode("utf-8")
