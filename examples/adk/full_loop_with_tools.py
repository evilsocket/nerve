import asyncio
from typing import Annotated

from nerve.models import Configuration
from nerve.runtime import logging
from nerve.runtime.agent import Agent


# annotate parameters to describe them to the agent
# async is optional in this case, but supported
async def get_capital(place: Annotated[str, "The place to get the capital of"]) -> str:
    """Get the capital of a given place."""

    # you can manually set the task as complete when a target objective is reached
    # from nerve.runtime import state
    # state.set_task_complete()

    return place.upper()


async def main():
    # pass debug=True to get more verbose logging
    logging.init(debug=False)

    agent = Agent.create(
        "openai/gpt-4o",  # the model to use
        Configuration(
            # prompts support jinja2 templating
            agent="You are a helpful assistant. Your task is complete when you have the capital of the place.",
            task="What is the capital of {{ place }}?",
            # add builtin tooling by namespace
            using=[
                "reasoning",  # to allow the agent to make analytical decisions
                "task",  # to allow the agent to set the task as complete autonomously
            ],
            tools=[get_capital],  # easily add custom tooling
        ),
    )

    # run until done or max steps reached or timeout
    await agent.run(start_state={"place": "Spain"}, max_steps=100, timeout=10)


if __name__ == "__main__":
    asyncio.run(main())
