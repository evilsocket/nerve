"""
Read-only access primitives to the local filesystem.
"""

import os
from typing import Annotated

from nerve.tools.utils import maybe_text, path_acl

# for docs
EMOJI = "📂"

# if set, the agent will only have access to these paths
jail: list[str] = []

# TODO: fix this missed interpolation:
# "\u001b[31m\u001b[1m[04-17-25 14:04:03] ERROR\u001b[0m list_folder_contents: access to path /Users/evilsocket/.nerve/agents/eval-regex/cases is not allowed, only allowed paths are: ['{{ CASE_PATH }}']",

def list_folder_contents(
    path: Annotated[str, "The path to the folder to list"],
) -> str:
    """List the contents of a folder on disk."""

    path_acl(path, jail)

    # The rationale here is that because of training data, models can
    # understand an "ls -la" output better than any custom output format
    # I could generate manually, so we just use the "ls -la" command to
    # list the contents of the folder.
    return os.popen(f"ls -la {path}").read()


def read_file(path: Annotated[str, "The path to the file to read"]) -> str | bytes:
    """Read the contents of a file from disk."""

    path_acl(path, jail)

    with open(path, "rb") as f:
        return maybe_text(f.read())
