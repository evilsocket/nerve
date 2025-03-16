"""
Write primitives to the local filesystem.
"""

import os
from pathlib import Path
from typing import Annotated

# for docs
EMOJI = "📂"

# if set, the agent will only have access to these paths
jail: list[str] = []


# TODO: abstract and centralize jail system
def _path_allowed(path_to_check: str) -> bool:
    if not jail:
        return True

    # https://stackoverflow.com/questions/3812849/how-to-check-whether-a-directory-is-a-sub-directory-of-another-directory
    path = Path(path_to_check).resolve().absolute()
    for allowed_path in jail:
        allowed = Path(allowed_path).resolve().absolute()
        if path == allowed or allowed in path.parents:
            return True

    return False


def _path_acl(path_to_check: str) -> None:
    if not _path_allowed(path_to_check):
        raise ValueError(f"access to path {path_to_check} is not allowed, only allowed paths are: {jail}")


def _maybe_text(output: bytes) -> str | bytes:
    try:
        return output.decode("utf-8").strip()
    except UnicodeDecodeError:
        return output


def create_file(
    path: Annotated[str, "The path to the file to create"],
    content: Annotated[
        str | None, "The content to write to the file, if not provided, the file will be created empty"
    ] = None,
) -> str:
    """Create a file on disk, if the file already exists, it will be overwritten."""

    _path_acl(path)

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

    _path_acl(path)

    os.remove(path)
    return f"File {path} deleted."
