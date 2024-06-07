#!/usr/bin/env python3
import asyncio
import pathlib
import random
import typing as t

import click
from loguru import logger
from state import Actions, ActionsList, State
from task import get_task_main_story

import rigging as rg
from rigging import logging


async def agent_loop(state: State) -> State:
    async def parse_actions(chat: rg.Chat) -> t.Optional[rg.Chat]:
        parsed: list[Actions] = []
        for action_cls in ActionsList:
            action = chat.last.try_parse(action_cls)
            if action is not None:
                parsed.append(action)  # type: ignore

        if not parsed:
            logger.warning("model didn't provide any valid actions")
            return None

        parsed = t.cast(list[Actions], [p.model for p in chat.last.parts])
        if len(parsed) > state.max_actions:
            # TODO: execute sequentially?
            logger.warning(
                f"model provided multiple actions, taking just the first: {parsed}")

        state.next_actions = parsed[: state.max_actions]
        return None

    while not state.result:
        await state.base_chat.fork(state.get_prompt()).then(parse_actions).arun()
        await state.step()

    return state


async def main(
    generator_id: str, max_iterations: int, parallel_agents: int, max_actions: int, max_tokens: int
) -> None:
    if parallel_agents > 1:
        logger.success(f"starting with {parallel_agents} agents")

    logger.success(f"using '{generator_id}'")

    generator = rg.get_generator(generator_id)
    base_chat = generator.chat(
        [{"role": "system", "content": get_task_main_story()}],
    ).with_(max_tokens=max_tokens)

    for i in range(max_iterations):
        logger.debug(f"Starting task ...")

        states: list[State] = [
            State(id=i, max_actions=max_actions, base_chat=base_chat.with_(
                temperature=random.uniform(0.25, 1)))
            for i in range(parallel_agents)
        ]
        for state in states:
            await state.prep()

        loops = [asyncio.create_task(agent_loop(state)) for state in states]
        _, pending = await asyncio.wait(loops, return_when=asyncio.FIRST_COMPLETED)

        for task in pending:
            task.cancel()

    logger.debug("Finished task.")


@click.command()
@click.option(
    "-g",
    "--generator-id",
    type=str,
    default="ollama/llama3",
    required=True,
    help="Rigging generator identifier (gpt-4, mistral/mistral-medium, etc.)",
)
@click.option(
    "-i",
    "--max-iterations",
    type=int,
    default=100,
    help="Maximum number of iterations",
)
@click.option(
    "-p",
    "--parallel-agents",
    type=int,
    default=1,
    help="Number of parallel agents",
)
@click.option(
    "-m",
    "--max-actions",
    type=int,
    default=3,
    help="Maximum number of actions allowed per generation round",
)
@click.option(
    "-m",
    "--max-tokens",
    type=int,
    default=4096,
    help="Maximum number of context tokens",
)
@click.option(
    "--log-level",
    type=click.Choice(logging.LogLevelList),
    default="info",
)
@click.option("--log-file", type=click.Path(path_type=pathlib.Path), default=None)
@click.option(
    "--log-file-level",
    type=click.Choice(logging.LogLevelList),
    default="trace",
)
def cli(
    generator_id: str,
    max_iterations: int,
    parallel_agents: int,
    max_actions: int,
    log_level: logging.LogLevelLiteral,
    log_file: pathlib.Path,
    log_file_level: logging.LogLevelLiteral,
) -> None:
    logging.configure_logging(log_level, log_file, log_file_level)
    asyncio.run(main(generator_id, max_iterations,
                parallel_agents, max_actions))


if __name__ == "__main__":
    cli()
