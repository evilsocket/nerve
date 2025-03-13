import typer

import nerve
from nerve.cli.agents import cli as agents_cli
from nerve.cli.create import cli as create_cli
from nerve.cli.namespaces import cli as namespaces_cli
from nerve.cli.replay import cli as replay_cli
from nerve.cli.run import cli as run_cli

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)

cli.add_typer(agents_cli)
cli.add_typer(create_cli)
cli.add_typer(run_cli)
cli.add_typer(replay_cli)
cli.add_typer(namespaces_cli)

# TODO Workflows 2.0: shared events bus (IPC and network) for multiple agents (one input variable = one event).
# Tools:
# advertise: ask the agent to advertise itself on the events bus
# request: allow the agent to request something from another agent
# respond: allow the agent to respond to a request
# at every step the agent goes requests = bus.query()
# TODO: browser-use: each interactable html generates tools at runtime.


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    help="Print the version and exit.",
)
def version() -> None:
    import platform
    import sys

    print(f"platform: {platform.system().lower()} ({platform.machine()})")
    print(f"python:   {sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}")
    print(f"nerve:    {nerve.__version__}")
