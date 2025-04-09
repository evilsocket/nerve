import pathlib
import time

from loguru import logger

import nerve.runtime.state as state
from nerve.generation import WindowStrategy
from nerve.generation.conversation import FullHistoryStrategy
from nerve.models import Workflow
from nerve.runtime.agent import Agent
from nerve.runtime.shell import Shell

IS_ACTIVE: bool = False


class Flow:
    def __init__(
        self,
        actors: list[Agent],
        workflow: Workflow | None = None,
        max_steps: int = 500,
        max_cost: float = 10.0,
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
        self.curr_step: int = 1
        # optional max steps to run
        self.max_steps: int = max_steps
        # max cost to run the flow
        self.max_cost: float = max_cost
        # optional timeout to run the flow
        self.timeout: int | None = timeout
        # start time of the flow
        self.started_at: float | None = None
        # interactive shell
        self.shell: Shell = Shell()

    @classmethod
    async def build(
        cls,
        actors: list[Agent],
        max_steps: int = 500,
        max_cost: float = 10.0,
        timeout: int | None = None,
        start_state: dict[str, str] | None = None,
    ) -> "Flow":
        if start_state:
            state.update_variables(start_state)

        return cls(actors=actors, max_steps=max_steps, max_cost=max_cost, timeout=timeout)

    @classmethod
    async def from_path(
        cls,
        input_path: pathlib.Path,
        window_strategy: WindowStrategy = FullHistoryStrategy(),
        max_steps: int = 500,
        max_cost: float = 10.0,
        timeout: int | None = None,
        start_state: dict[str, str] | None = None,
    ) -> "Flow":
        workflow = Workflow.from_path(input_path)
        root_path = input_path if input_path.is_dir() else input_path.parent

        actors = []
        for actor_name, actor in workflow.actors.items():
            # determine actor task file
            task_file_path = (root_path / actor_name).with_suffix(".yml")
            if not task_file_path.exists():
                task_file_path = root_path / actor_name

            actors.append(await Agent.create_from_file(actor.generator, task_file_path, window_strategy))

        if start_state:
            state.update_variables(start_state)

        return cls(
            actors=actors,
            workflow=workflow,
            max_steps=max_steps,
            max_cost=max_cost,
            timeout=timeout,
        )

    async def _setup_if_needed(self, task_override: str | None = None) -> None:
        if self.started_at is None:
            logger.debug("setting started at")
            self.started_at = time.time()

        if self.curr_actor is None:
            logger.debug(f"setting curr actor to {self.curr_actor_idx}")
            self.curr_actor = self.actors[self.curr_actor_idx]

            if task_override:
                logger.info(f"🎯 setting task: {task_override}")
                self.curr_actor.configuration.task = task_override

            state.set_tools({tool.__name__: tool for tool in self.curr_actor.runtime.tools})
            state.set_defaults(self.curr_actor.configuration.defaults)
            state.on_task_started(self.curr_actor)

    async def step(self) -> None:
        logger.debug("flow.step")

        await self._setup_if_needed()

        if self.done():
            logger.debug("flow done")
            state.on_event("flow_complete", {"steps": self.curr_step - 1, "usage": state.get_usage()})
            return

        state.on_event("step_started", {"step": self.curr_step, "usage": state.get_usage()})

        step_usage = await self.curr_actor.step()  # type: ignore
        logger.debug(f"step usage: {step_usage}")

        # increment total usage
        state.update_usage(step_usage)
        state.on_event("step_complete", {"step": self.curr_step, "usage": state.get_usage()})

        if state.is_active_task_done():
            logger.debug(f"task {self.curr_actor.runtime.name} complete")  # type: ignore
            self.curr_actor_idx += 1
            self.curr_actor = None
            state.reset()

        self.curr_step += 1

    def done(self) -> bool:
        if self.curr_actor_idx >= len(self.actors):
            logger.debug("all actors done")
            return True

        if self.max_steps > 0 and self.curr_step > self.max_steps:
            logger.debug("max steps reached")
            state.on_max_steps_reached()
            return True

        usage = state.get_usage()
        if self.max_cost > 0 and usage.cost is not None and usage.cost > self.max_cost:
            logger.debug("max cost reached")
            state.on_max_cost_reached()
            return True

        if self.timeout is not None and self.started_at is not None and time.time() - self.started_at > self.timeout:
            logger.debug("timeout reached")
            state.on_timeout()
            return True

        return False

    async def _reset(self) -> None:
        logger.debug("flow reset")
        state.reset()
        await self.shell.reset()
        self.curr_actor_idx = 0
        self.curr_actor = None

    async def run(self, task_override: str | None = None) -> None:
        state.on_event(
            "flow_started",
            {
                "flow": self,
                "state": state.as_dict(),
            },
        )

        while not self.done():
            await self._setup_if_needed(task_override)

            if self.curr_actor:
                logger.debug("interact if needed")
                await self.shell.interact_if_needed(self.curr_actor)
            else:
                logger.debug("no actor, can't interact")

            await self.step()

            # in interactive mode, we reset and restart when we're done
            # to let the user quit or change the task
            if self.done() and state.is_interactive():
                await self._reset()

        logger.debug("flow complete")

        state.on_event(
            "flow_complete",
            {
                "workflow": self.workflow,
                "steps": self.curr_step - 1,
                "usage": state.get_usage(),
                "state": state.as_dict(),
            },
        )
