import pathlib
import typing as t

import natsort
import requests
import typer
from termcolor import colored

import nerve
from nerve.defaults import (
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
    help="List locally installed agents and available ones from the awesomeagents.ai index.",
)
def agents(
    path: t.Annotated[
        pathlib.Path,
        typer.Argument(help="Path to the agent or workflow to create"),
    ] = DEFAULT_AGENTS_LOAD_PATH,
    offline: t.Annotated[
        bool,
        typer.Option("--offline", "-o", help="Only show installed agents."),
    ] = False,
) -> None:
    print(f"🧠 nerve v{nerve.__version__}")

    _show_installed_agents(path)

    if not offline:
        agents = _fetch_awesome_agents()
        if agents:
            _show_awesome_agents(agents)


def _fetch_awesome_agents() -> list[dict[str, t.Any]] | None:
    try:
        response = requests.get("https://api.awesomeagents.ai/index.json")
        response.raise_for_status()
        agents = response.json()
        return [agent for agent in agents if "nerve" in agent.get("stack", [])]
    except Exception:
        return None


def _show_awesome_agents(agents: list[dict[str, t.Any]]) -> None:
    print("🔍 Available from the index:\n")
    for agent in agents:
        repo = agent["repo"]
        # Extract username/repo from the repository URL
        repo_parts = repo.split("/")
        if len(repo_parts) >= 2:
            username_repo = f"{repo_parts[-2]}/{repo_parts[-1]}"
        else:
            username_repo = repo
        print(f"     {colored(username_repo, 'white', attrs=['bold'])} - {agent['description']}")

    print()


def _show_installed_agents(path: pathlib.Path) -> None:
    anything = False

    if path.exists() and path.is_dir():
        print()
        print(f"📁 Installed in {path.absolute()}:\n")

        items = []
        for item in path.iterdir():
            items.append(item)

        for item in natsort.natsorted(items):
            if Workflow.is_workflow(item):
                workflow = Workflow.from_path(item)
                print(
                    f"     {colored(item.name, 'white', attrs=['bold'])} "
                    + colored("<workflow>", "blue")
                    + f" - {workflow.description}"
                )
                anything = True
            elif Configuration.is_agent_config(item):
                config = Configuration.from_path(item)
                print(
                    f"     {colored(item.name, 'white', attrs=['bold'])} "
                    + colored("<agent>", "green")
                    + f" - {config.description}"
                )
                anything = True

        print()

    if not anything:
        print(colored(f"No agents or workflows found in {path}", "light_grey"))
