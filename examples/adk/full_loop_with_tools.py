import asyncio
from typing import Annotated

import httpx

from nerve.models import Configuration
from nerve.runtime import logging
from nerve.runtime.agent import Agent


# Annotate functions and parameters to describe them to the agent.
async def get_current_weather(location: Annotated[str, "The city and state, e.g. San Francisco, CA"]) -> str:
    """Get the current weather in a given location."""

    try:
        async with httpx.AsyncClient() as client:
            r = await client.get("https://wttr.in/" + location)
            return r.text
    except Exception as e:
        # let the agent know what happened
        return f"ERROR: {e}"


async def main():
    # pass level='DEBUG' to get more verbose logging or level='SUCCESS' to get quieter logging
    logging.init(level="INFO")

    agent = await Agent.create(
        "openai/gpt-4o",  # the model to use
        Configuration(
            agent="You are a helpful assistant.",
            task="What is the weather in {{ place }}?",
            using=[
                "task",  # to allow the agent to set the task as complete autonomously
            ],
            tools=[get_current_weather],
        ),
    )

    # run until done or max steps reached or max cost reached or timeout
    await agent.run(start_state={"place": "Rome"}, max_steps=100, max_cost=10.0, timeout=10)


if __name__ == "__main__":
    asyncio.run(main())
