"""
Read-only access primitives to the local filesystem.
"""

import os
from pathlib import Path
from typing import Annotated

# if set, the agent will only have access to these paths
jail: list[str] = []


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


def list_folder_contents(
    path: Annotated[str, "The path to the folder to list"],
) -> str:
    """List the contents of a folder on disk."""

    _path_acl(path)

    # The rationale here is that because of training data, models can
    # understand an "ls -la" output better than any custom output format
    # I could generate manually, so we just use the "ls -la" command to
    # list the contents of the folder.
    return os.popen(f"ls -la {path}").read()


def read_file(path: Annotated[str, "The path to the file to read"]) -> str:
    """Read the contents of a file from disk."""

    _path_acl(path)

    with open(path) as f:
        return f.read()
