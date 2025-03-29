import asyncio

from nerve.models import Configuration
from nerve.runtime import logging
from nerve.runtime.agent import Agent


async def main():
    # pass level='DEBUG' to get more verbose logging or level='SUCCESS' to get quieter logging
    logging.init(level="INFO")

    agent = await Agent.create(
        "openai/gpt-4o",  # the model to use
        Configuration(
            agent="You are a helpful assistant that solves math problems.",
            task="What is the result of: 3 + 3?",
        ),
    )

    # perform one step
    await agent.step()


if __name__ == "__main__":
    asyncio.run(main())
