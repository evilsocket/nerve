"""
Let the agent execute shell commands.
"""

import subprocess
from typing import Annotated

from nerve.tools.utils import maybe_text

# for docs
EMOJI = "💻"


# TODO: if both filesystem and shell are used, shell can be used to bypass the filesystem jailing system. find a way to either prevent it, or communicate it.
def shell(
    command: Annotated[str, "The shell command to execute"],
) -> str | bytes:
    """Execute a shell command on the local computer and return the output. Non interactive shell with a timeout of 30 seconds."""

    result = subprocess.run(command, shell=True, capture_output=True, timeout=30)

    raw_output = result.stdout or b""

    if result.returncode != 0:
        raw_output += b"\nEXIT CODE: " + str(result.returncode).encode("utf-8")

    if result.stderr:
        if result.returncode != 0:
            raw_output += b"\nERROR: " + result.stderr
        else:
            raw_output += b"\n" + result.stderr

    return maybe_text(raw_output)
