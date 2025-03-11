import pathlib

import typer
from loguru import logger
from termcolor import colored

from nerve.models import Configuration, Workflow


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
