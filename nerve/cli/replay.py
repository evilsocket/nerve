import asyncio
import json
import pathlib
import typing as t

import typer
from loguru import logger

import nerve
from nerve.runtime import logging
from nerve.runtime.events import Event

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=True,
    help="Replay a trace file.",
)
def play(
    trace_path: t.Annotated[
        pathlib.Path,
        typer.Argument(help="Trace file to replay"),
    ] = pathlib.Path("trace.jsonl"),
    fast: t.Annotated[
        bool,
        typer.Option("--fast", "-f", help="Do not sleep between events"),
    ] = False,
) -> None:
    logging.init(level="INFO")
    logger.info(f"🧠 nerve v{nerve.__version__}")

    asyncio.run(replay(trace_path, fast))


async def replay(
    trace_path: pathlib.Path,
    fast: bool,
) -> None:
    logger.info(f"▶️  replaying {trace_path} ...")

    with open(trace_path) as f:
        prev: Event | None = None

        for line in f:
            event = Event(**json.loads(line))

            if prev and not fast:
                delay = event.timestamp - prev.timestamp
                await asyncio.sleep(delay)

            logging.log_event_to_terminal(event)

            prev = event
