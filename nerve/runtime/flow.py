import os
import pathlib
import time

from loguru import logger

import nerve.runtime.state as state
from nerve.generation import Usage, WindowStrategy
from nerve.generation.conversation import FullHistoryStrategy
from nerve.models import Workflow
from nerve.runtime.agent import Agent

IS_ACTIVE: bool = False


class Flow:
    def __init__(
        self,
        actors: list[Agent],
        workflow: Workflow | None = None,
        max_steps: int = 500,
        timeout: int | None = None,
    ):
        global IS_ACTIVE

        if IS_ACTIVE:
            raise RuntimeError("A flow is already running")

        IS_ACTIVE = True

        # all actors in the flow
        self.actors = actors
        # workflow definition if set or None if we're running a single agent
        self.workflow = workflow
        # current active agent in the flow
        self.curr_actor_idx: int = 0
        self.curr_actor: Agent | None = None
        # current step from the beginning of the flow
        self.curr_step: int = 0
        # total token usage accumulated over each step
        self.token_usage: Usage = Usage(prompt_tokens=0, completion_tokens=0, total_tokens=0)
        # optional max steps to run
        self.max_steps: int = max_steps
        # optional timeout to run the flow
        self.timeout: int | None = timeout
        # start time of the flow
        self.started_at: float | None = None

    @classmethod
    def build(
        cls,
        actors: list[Agent],
        max_steps: int = 500,
        timeout: int | None = None,
        start_state: dict[str, str] | None = None,
    ) -> "Flow":
        if start_state:
            state.update_variables(start_state)

        return cls(actors=actors, max_steps=max_steps, timeout=timeout)

    @classmethod
    def from_path(
        cls,
        input_path: pathlib.Path,
        window_strategy: WindowStrategy = FullHistoryStrategy(),
        max_steps: int = 500,
        timeout: int | None = None,
        start_state: dict[str, str] | None = None,
    ) -> "Flow":
        workflow = Workflow.from_path(input_path)
        root_path = input_path if input_path.is_dir() else input_path.parent

        actors = []
        for actor_name, actor in workflow.flow.items():
            # determine actor task file
            task_file_path = (root_path / actor_name).with_suffix(".yml")
            if not task_file_path.exists():
                task_file_path = root_path / actor_name

            actors.append(Agent.create_from_file(actor.generator, task_file_path, window_strategy))

        if start_state:
            state.update_variables(start_state)

        return cls(
            actors=actors,
            workflow=workflow,
            max_steps=max_steps,
            timeout=timeout,
        )

    async def step(self) -> None:
        if self.started_at is None:
            self.started_at = time.time()

        if self.curr_actor is None:
            self.curr_actor = self.actors[self.curr_actor_idx]
            state.on_task_started(self.curr_actor)
            os.chdir(self.curr_actor.runtime.working_dir)

        if self.done():
            state.on_event("flow_complete", {"steps": self.curr_step})
            return

        state.on_event("step_started", {"step": self.curr_step, "token_usage": self.token_usage})

        step_usage = await self.curr_actor.step()
        logger.debug(f"step usage: {step_usage}")

        # increment total usage
        self.token_usage.prompt_tokens += step_usage.prompt_tokens
        self.token_usage.completion_tokens += step_usage.completion_tokens
        self.token_usage.total_tokens += step_usage.total_tokens

        state.on_event("step_complete", {"step": self.curr_step, "token_usage": self.token_usage})

        if state.is_active_task_done():
            logger.debug(f"task {self.curr_actor.runtime.name} complete")
            self.curr_actor_idx += 1
            self.curr_actor = None
            state.reset()

        self.curr_step += 1

    def done(self) -> bool:
        if self.curr_actor_idx >= len(self.actors):
            return True

        if self.max_steps is not None and self.curr_step > self.max_steps:
            state.on_max_steps_reached()
            return True

        if self.timeout is not None and self.started_at is not None and time.time() - self.started_at > self.timeout:
            state.on_timeout()
            return True

        return False

    async def run(self) -> None:
        state.on_event(
            "flow_started",
            {
                "flow": self,
                "state": state.as_dict(),
            },
        )

        while not self.done():
            await self.step()

        state.on_event(
            "flow_complete",
            {
                "workflow": self.workflow,
                "steps": self.curr_step - 1,
                "usage": self.token_usage,
                "state": state.as_dict(),
            },
        )
