import pathlib
import shutil
import typing as t

import typer

import nerve
from nerve.defaults import (
    DEFAULT_AGENTS_LOAD_PATH,
)

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    help=f"Uninstall an agent or workflow from {DEFAULT_AGENTS_LOAD_PATH}",
)
def uninstall(
    name: t.Annotated[
        str,
        typer.Argument(help="Name of the agent or workflow to uninstall."),
    ],
    yes: t.Annotated[
        bool,
        typer.Option("--yes", "-y", help="Uninstall without asking for confirmation."),
    ] = False,
) -> None:
    print(f"🧠 nerve v{nerve.__version__}")

    # make sure no funny business is happening
    if ".." in name:
        print(f"🚨 {name} is not a valid agent or workflow name")
        exit(1)

    # get the path to the agent or workflow
    path = pathlib.Path(DEFAULT_AGENTS_LOAD_PATH) / name
    if not path.exists():
        print(f"❌ {name} does not exist in {DEFAULT_AGENTS_LOAD_PATH}")
        exit(1)

    # ask for confirmation
    if not yes:
        typer.confirm(
            f"⚠️  Are you sure you want to delete {name} from {DEFAULT_AGENTS_LOAD_PATH}? This action is irreversible.",
            abort=True,
        )

    # uninstall the agent or workflow
    shutil.rmtree(path)
    print(f"🧠 {name} uninstalled from {DEFAULT_AGENTS_LOAD_PATH}")
