import pkgutil

import typer
from termcolor import colored

import nerve
import nerve.tools.namespaces as ns_pkg

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    help="List all the available tools namespaces.",
)
def namespaces() -> None:
    print(f"🧠 nerve v{nerve.__version__}")
    print()

    for _, modname, _ in pkgutil.iter_modules(ns_pkg.__path__):
        if modname[0] != "_" and "test" not in modname:
            module = __import__(f"nerve.tools.namespaces.{modname}", fromlist=[""])

            doc = module.__doc__ or ""
            print(f"{module.EMOJI} {colored(modname, 'magenta')} - {colored(doc.strip(), 'dark_grey')}")
