"""
Provides tools for getting the current date and time and waiting for a given number of seconds.
"""

import time
from typing import Annotated

# for docs
EMOJI = "🕒"


def current_time_and_date() -> str:
    """Get the current date and time."""

    return time.strftime("%H:%M%p %Z on %b %d, %Y")


def wait(
    seconds: Annotated[int, "The number of seconds to wait"],
) -> None:
    """Wait for a given number of seconds."""

    time.sleep(seconds)
