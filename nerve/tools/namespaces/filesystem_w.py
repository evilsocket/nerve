"""
Write primitives to the local filesystem.
"""

import os
from typing import Annotated

from nerve.tools.utils import path_acl

# for docs
EMOJI = "📂"

# if set, the agent will only have access to these paths
jail: list[str] = []


def create_file(
    path: Annotated[str, "The path to the file to create"],
    content: Annotated[
        str | None, "The content to write to the file, if not provided, the file will be created empty"
    ] = None,
) -> str:
    """Create a file on disk, if the file already exists, it will be overwritten."""

    path_acl(path, jail)

    response = ""

    # ensure parent directory exists
    parent_dir = os.path.dirname(path)
    if parent_dir and not os.path.exists(parent_dir):
        os.makedirs(parent_dir)
        response += f"Created parent directory {parent_dir}.\n"

    exists = os.path.exists(path)

    with open(path, "w") as f:
        f.write(content or "")

    if exists:
        response += f"File {path} updated.\n"
    else:
        response += f"File {path} created.\n"

    return response


def delete_file(path: Annotated[str, "The path to the file to delete"]) -> str:
    """Delete a file from disk."""

    path_acl(path, jail)

    os.remove(path)
    return f"File {path} deleted."
