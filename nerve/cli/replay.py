import asyncio
import json
import pathlib

from loguru import logger

from nerve.runtime import logging
from nerve.runtime.events import Event


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
