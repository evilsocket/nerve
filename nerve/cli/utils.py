import pathlib

import typer
from loguru import logger

from nerve.cli.defaults import DEFAULT_AGENTS_LOAD_PATH


def _resolve_input_path(input_path: pathlib.Path) -> pathlib.Path:
    # check if input_path exists
    if not input_path.exists():
        input_path_with_yaml = input_path.with_suffix(".yml")
        if input_path_with_yaml.exists():
            input_path = input_path_with_yaml

        elif not input_path.is_absolute():
            # check if it exists as part of the $HOME/.nerve/agents directory
            in_home = DEFAULT_AGENTS_LOAD_PATH / input_path
            if in_home.exists():
                input_path = in_home

            in_home_with_yaml = in_home.with_suffix(".yml")
            if in_home_with_yaml.exists():
                input_path = in_home_with_yaml

        if not input_path.exists():
            logger.error(f"path '{input_path}' does not exist")
            raise typer.Abort()

    return input_path
