import asyncio
import sys
from typing import Annotated

from loguru import logger

from nerve.models import Configuration
from nerve.runtime import logging, state
from nerve.runtime.agent import Agent


async def provide_answer(result: Annotated[float, "The result of the math problem"]) -> None:
    """Provide the answer to the question."""

    # get the expression from the state
    expression = state.get_variable("expression")
    # evaluate it in python to get the correct result
    expected_result = eval(expression)
    # compare the result with the expected result
    if abs(result - expected_result) < 0.002:
        logger.info(f"{expression} = {result}")
        state.set_task_complete()
    else:
        logger.error(f"{expression} != {result} (expected {expected_result})")
        state.set_task_failed(f"The result of {expression} is not {expected_result}")


async def main():
    # pass level='DEBUG' to get more verbose logging or level='SUCCESS' to get quieter logging
    logging.init(level="INFO")

    if len(sys.argv) != 2:
        logger.error("Usage: python single_step.py <expression>")
        sys.exit(1)

    agent = Agent.create(
        "openai/gpt-4o",  # the model to use
        Configuration(
            agent="You are a helpful assistant that solves math problems.",
            task="What is the result of: {{ expression }}?",
            tools=[provide_answer],
        ),
        start_state={"expression": sys.argv[1]},
    )

    # perform one step
    await agent.step()


if __name__ == "__main__":
    asyncio.run(main())
