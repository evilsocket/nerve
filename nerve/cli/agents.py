import asyncio
import pathlib
import typing as t

import typer
from loguru import logger
from termcolor import colored

import nerve
from nerve.cli.defaults import (
    DEFAULT_AGENTS_LOAD_PATH,
)
from nerve.models import Configuration, Workflow

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    help="List the agents available locally in $HOME/.nerve/agents or a custom path.",
)
def agents(
    path: t.Annotated[
        pathlib.Path,
        typer.Argument(help="Path to the agent or workflow to create"),
    ] = DEFAULT_AGENTS_LOAD_PATH,
) -> None:
    print(f"🧠 nerve v{nerve.__version__}")

    asyncio.run(show_agents(path))


async def show_agents(path: pathlib.Path) -> None:
    if not path.exists():
        logger.error(f"Path {path} does not exist")
        raise typer.Exit(1)

    if not path.is_dir():
        logger.error(f"Path {path} is not a directory")
        raise typer.Exit(1)

    print()
    print(f"📁 {path.absolute()}")

    anything = False
    for item in path.iterdir():
        if Workflow.is_workflow(item):
            print(f"   {item.name} " + colored("<workflow>", "blue"))
            anything = True
        elif Configuration.is_agent_config(item):
            print(f"   {item.name} " + colored("<agent>", "green"))
            anything = True

    if not anything:
        print(colored("no agents or workflows found", "light_grey"))
