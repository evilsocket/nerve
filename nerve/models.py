import pathlib
import typing as t
from enum import Enum

from pydantic import BaseModel, Field
from pydantic_yaml import parse_yaml_raw_as


class Mode(str, Enum):
    """
    Running mode.
    """

    INTERACTIVE = "interactive"
    AUTOMATIC = "automatic"


class Status(str, Enum):
    # used in interactive mode
    PAUSED = "paused"
    # the system is running (trying to solve the task)
    RUNNING = "running"
    # the task was completed successfully
    COMPLETED = "completed"
    # the task failed
    FAILED = "failed"

    def is_done(self) -> bool:
        return self in [Status.COMPLETED, Status.FAILED]


class Tool(BaseModel):
    """
    A tool is a function that can be called by the agent.
    """

    class Argument(BaseModel):
        """
        An argument is a parameter of the tool.
        """

        name: str
        description: str
        example: str

    # if path is set, it'll be loaded from a python file
    path: str | None = None

    # yaml description of the tool
    name: str = ""
    description: str = ""
    arguments: list[Argument] = []
    complete_task: bool = False
    mime: str | None = None
    tool: str | None = None


class Configuration(BaseModel):
    """
    Configuration for an agent determining its "identity", task and capabilities.
    """

    # legacy field used to detect if the user is loading a legacy file
    system_prompt: str | None = Field(default=None, exclude=True)

    # used for versioning the agents
    version: str = "1.0.0"
    # the system prompt, the agent identity
    agent: str | None = None
    # the main agent task
    task: str | None = None
    # builtin namespaces
    using: list[str] = []
    # jail mechanism for each namespace
    jail: dict[str, list[str]] = {}
    # custom tooling
    tools: list[Tool | t.Callable[..., t.Any]] = []

    @staticmethod
    def is_agent_config(input_path: pathlib.Path) -> bool:
        try:
            _ = Configuration.from_path(input_path)
            return True
        except Exception:
            return False

    @classmethod
    def from_path(cls, input_path: pathlib.Path) -> "Configuration":
        if input_path.is_dir():
            for option in ("task.yml", "agent.yml"):
                sub_path = input_path / option
                if sub_path.exists():
                    input_path = sub_path
                    break

        with open(input_path) as f:
            return parse_yaml_raw_as(cls, f.read())

    @classmethod
    def from_yml(cls, config_yml: str) -> "Configuration":
        return parse_yaml_raw_as(cls, config_yml)

    @property
    def is_legacy(self) -> bool:
        return self.system_prompt is not None


class Workflow(BaseModel):
    """
    A workflow is a collection of agents that are executed sequentially.
    """

    class Actor(BaseModel):
        """
        An actor is an agent that is part of the workflow.
        """

        generator: str

    name: str
    description: str
    flow: dict[str, Actor]

    @staticmethod
    def is_workflow(input_path: pathlib.Path) -> bool:
        try:
            _ = Workflow.from_path(input_path)
            return True
        except Exception:
            return False

    @classmethod
    def from_path(
        cls,
        input_path: pathlib.Path,
    ) -> "Workflow":
        if input_path.is_dir():
            input_path = input_path / "workflow.yml"

        with open(input_path) as f:
            return parse_yaml_raw_as(cls, f.read())


__all__ = ["Mode", "Status", "Tool", "Configuration", "Workflow"]
