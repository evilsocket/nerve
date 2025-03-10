"""
Let the agent execute shell commands.
"""

import subprocess
from typing import Annotated


def _maybe_text(output: bytes) -> str | bytes:
    try:
        return output.decode("utf-8").strip()
    except UnicodeDecodeError:
        return output


def execute_shell_command(
    command: Annotated[str, "The shell command to execute"],
) -> str | bytes:
    """Execute a shell command on the local computer and return the output with a timeout of 30 seconds."""

    result = subprocess.run(command, shell=True, capture_output=True, timeout=30)

    raw_output = result.stdout or b""

    if result.returncode != 0:
        raw_output += b"\nEXIT CODE: " + str(result.returncode).encode("utf-8")

    if result.stderr:
        raw_output += b"\nERROR: " + result.stderr

    return _maybe_text(raw_output)
