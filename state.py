import abc
import json
import typing as t
from dataclasses import dataclass, field

from loguru import logger
from pydantic import StringConstraints
from task import (create_task_context, execute_task_action, get_task_description,
                  validate_task_completion)

import rigging as rg

str_strip = t.Annotated[str, StringConstraints(strip_whitespace=True)]


class Action(rg.Model, abc.ABC):
    @abc.abstractmethod
    async def run(self, state: "State") -> str:
        ...


class UpdateGoal(Action):
    goal: str_strip

    @classmethod
    def xml_example(cls) -> str:
        return UpdateGoal(goal="My new goal").to_pretty_xml()

    async def run(self, state: "State") -> str:
        logger.success(f"[{state.id}] new goal '{self.goal}'")
        state.goals.append(self.goal)
        return "Goal updated."


class SaveMemory(Action):
    key: str_strip = rg.attr()
    content: str_strip

    @classmethod
    def xml_example(cls) -> str:
        return SaveMemory(key="my-note", content="Lots of custom data\nKeep this for later.").to_pretty_xml()

    async def run(self, state: "State") -> str:
        logger.success(f"[{state.id}] storing '{self.key}': {self.content}")
        state.memories[self.key] = self.content
        return f"Stored '{self.key}'."


class RecallMemory(Action):
    key: str_strip

    @classmethod
    def xml_example(cls) -> str:
        return RecallMemory(key="last-thoughts").to_pretty_xml()

    async def run(self, state: "State") -> str:
        value = state.memories.get(self.key, "Not found.")
        logger.success(f"[{state.id}] recalling '{self.key}': {value}")
        return value


class DeleteMemory(Action):
    key: str_strip

    @classmethod
    def xml_example(cls) -> str:
        return DeleteMemory(key="my-note").to_pretty_xml()

    async def run(self, state: "State") -> str:
        logger.success(f"[{state.id}] forgetting '{self.key}'")
        state.memories.pop(self.key, None)
        return f"Forgot '{self.key}'."


class PinToTop(Action):
    content: str_strip

    @classmethod
    def xml_example(cls) -> str:
        return PinToTop(content="This is the auth token: 1234").to_pretty_xml()

    async def run(self, state: "State") -> str:
        logger.success(f"[{state.id}] pinning '{self.content.strip()}'")
        state.pins.append(self.content)
        state.pins = state.pins[:state.max_pins]
        return "Pinned."


class TryCommand(Action):
    content: str_strip

    @classmethod
    def xml_example(cls) -> str:
        return TryCommand(content="whoami | grep abc").to_pretty_xml()

    async def run(self, state: "State") -> str:
        return await execute_task_action(state.task_context, self.content)


class PerformTaskCompletion(Action):
    info: str_strip

    @classmethod
    def xml_example(cls) -> str:
        return PerformTaskCompletion(info="I HAVE DONE MASTER").to_pretty_xml()

    async def run(self, state: "State") -> str:
        if validate_task_completion(state):
            logger.warning("TASK COMPLETED")
            state.finish(state.toJSON())
            return "Success"

        return "invalid solution, try again."


Actions = t.Union[UpdateGoal, SaveMemory, RecallMemory,
                  DeleteMemory, PinToTop, TryCommand, PerformTaskCompletion]
ActionsList: list[type[Actions]] = [
    UpdateGoal,
    SaveMemory,
    RecallMemory,
    DeleteMemory,
    PinToTop,
    TryCommand,
    PerformTaskCompletion,
]


@dataclass
class State:
    # Required
    id: int
    max_actions: int
    max_tokens: int
    max_pins: int
    max_history: int
    base_chat: rg.PendingChat

    # Progress
    result: t.Optional[str] = ""

    # Task
    task_context: t.Optional[t.Any] = None
    task_description: str = ""

    # Core
    goals: list[str] = field(default_factory=list)
    next_actions: list[Actions] = field(default_factory=list)

    # Context
    pins: list[str] = field(default_factory=list)
    memories: dict[str, str] = field(default_factory=dict)
    history: list[tuple[Actions, str]] = field(default_factory=list)

    # https://stackoverflow.com/questions/3768895/how-to-make-a-class-json-serializable
    def toJSON(self):
        return json.dumps(
            self,
            default=lambda o: str(o),
            sort_keys=True,
            indent=4)

    def finish(self, result) -> None:
        logger.info("state::finish")
        self.result = result

    async def prep(self) -> None:
        self.task_context = await create_task_context()
        self.task_description = get_task_description()
        print()
        self.goals.append(f"complete the task.")

    async def step(self) -> None:
        logger.debug(f"processing {len(self.next_actions)} action(s)")
        for action in self.next_actions:
            self.history.append((action, await action.run(self)))
        self.next_actions.clear()

    def get_prompt(self) -> str:
        memories = "\n".join(self.memories.keys())
        previous_goals = "\n".join(
            self.goals[:-1] if len(self.goals) > 1 else [])
        current_goal = self.goals[-1]
        history = "\n---\n".join([h[0].to_pretty_xml() + "\n" + h[1]
                                 for h in self.history[-self.max_history:]])
        pinned = "\n".join(self.pins)
        return f"""\
# Context

<current-task-description>
{self.task_description}
</current-task-description>

<memories>
{memories or 'No memories yet.'}
</memories>

<last-{self.max_history}-actions>
{history or 'No actions taken yet'}
</last-{self.max_history}-actions>

<pinned>
{pinned or 'No pinned context yet.'}
</pinned>

# Goals

<previous-goals>
{previous_goals or 'No previous goals'}
</previous-goals>

<current-goal>
{current_goal}
</current-goal>

# Actions

You can take any 1 of the following actions in your response. Use the formats below.

## Memory

You can use the memory actions to store and retrieve long term information as you work. Use memories often to keep track of important information like your planning, analysis, important web responses, etc.

To store a memory:
{SaveMemory.xml_example()}

To retrieve a memory:
{RecallMemory.xml_example()}

To forget a memory:
{DeleteMemory.xml_example()}

## Pinned Context

In addition to long term memories, you can pin important short term information to be displayed before every future action (maximum of {self.max_pins}). Use this to keep data like tokens, passwords, usernames, endpoints, critical notes, etc.

To pin important information:
{PinToTop.xml_example()}

## Goal

When you believe you've accomplished your current goal:
{UpdateGoal.xml_example()}

## Commands

To execute a command via bash:
{TryCommand.xml_example()}

## Completion

When you have achieved your goal, report your state:
{PerformTaskCompletion.xml_example()}

---

# Guidance

- Do not use placeholder values in any of your actions.
- Recall your memories to help you make decisions.
- Experiment frequently and be adjust your approach to make progress.
- All actions are useful, be organized and methodical in your approach.
- Always assume you start in a new /bin/bash shell in the user home directory
- Add verbosity and failure information to your commands to assist with troubleshooting
- Prefer using full paths to files and directories
- Be creative with your solutions, but keep them simple
- Avoid repeating commands that have already been executed and failed
- Read output carefully to understand what errors occured and why
- Use the /tmp directory for any file write operations
- If a command you execute is not found, determine the linux distribution and install them using the proper package manager
- If you need to use the command 'sudo' before something, determine if you are root and only use sudo if you are not

Output a new action in your response. Prior action results are displayed above.
"""
