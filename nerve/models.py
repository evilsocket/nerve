import pathlib
import typing as t
from enum import Enum

from loguru import logger
from pydantic import AfterValidator, BaseModel, Field
from pydantic_yaml import parse_yaml_raw_as

import nerve


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


class Usage(BaseModel):
    cost: float | None = None
    prompt_tokens: int = 0
    completion_tokens: int = 0
    total_tokens: int = 0

    def __add__(self, other: "Usage") -> "Usage":
        return Usage(
            cost=(self.cost or 0) + (other.cost or 0),
            prompt_tokens=self.prompt_tokens + other.prompt_tokens,
            completion_tokens=self.completion_tokens + other.completion_tokens,
            total_tokens=self.total_tokens + other.total_tokens,
        )


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
        example: str = ""

    # if path is set, it'll be loaded from a python file
    path: str | None = None

    # yaml description of the tool
    name: str = ""
    description: str = ""
    arguments: list[Argument] = []
    complete_task: bool = False
    print: bool = False
    mime: str | None = None
    tool: str | None = None


def _check_required_version(required: str | None) -> str | None:
    if required:
        from packaging.requirements import Requirement

        try:
            # a version was specified, convert to a valid requirement
            if required[0].isdigit():
                required_str = f"nerve-adk>={required}"
            else:
                # a full expression was specified, use it as is
                required_str = f"nerve-adk{required}"
            req = Requirement(required_str)
        except Exception as e:
            logger.error(f"error parsing required version '{required}': {e}")
            raise

        if nerve.__version__ not in req.specifier:
            msg = f"required version {required} not satisfied by installed version {nerve.__version__}"
            logger.error(msg)
            raise ValueError(msg)

    return required


class Configuration(BaseModel):
    """
    Configuration for an agent determining its "identity", task and capabilities.
    """

    class MCPServer(BaseModel):
        """
        A MCP server is a server that implements the MCP protocol.
        """

        session_timeout: float = 5

        command: str = "python"
        args: list[str] = []
        env: dict[str, str] = {}
        # for SSE
        url: str | None = None
        headers: dict[str, t.Any] | None = None
        timeout: float = 5
        sse_read_timeout: float = 60 * 5

    class Limits(BaseModel):
        runs: int | None = None
        max_steps: int | None = None
        max_cost: float | None = None
        timeout: int | None = None

    # legacy field used to detect if the user is loading a legacy file
    system_prompt: str | None = Field(default=None, exclude=True)

    # optional generator
    generator: str | None = None
    # optional agent description
    description: str = ""
    # optional nerve version requirement
    requires: t.Annotated[str | None, AfterValidator(_check_required_version)] = None
    # used for versioning the agents
    version: str = "1.0.0"
    # the system prompt, the agent identity
    agent: str | None = None
    # the main agent task
    task: str | None = None
    # optional default values for the initial state
    defaults: dict[str, t.Any] = {}
    # builtin namespaces
    using: list[str] = []
    # jail mechanism for each namespace
    jail: dict[str, list[str]] = {}
    # MCP ( https://modelcontextprotocol.io/ ) servers.
    mcp: dict[str, MCPServer] = {}
    # optional limits
    limits: Limits | None = None
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

    def _get_inputs_from_string(self, string: str) -> set[str]:
        from jinja2 import Environment, meta

        return meta.find_undeclared_variables(Environment().parse(string))

    def get_inputs(self) -> dict[str, t.Any]:
        """
        Get the input names for the agent with their default values if set.
        """
        input_names = set()
        tools_names = [t.name if isinstance(t, Tool) else t.__name__ for t in self.tools]

        # from the system prompt
        if self.agent:
            for input_name in self._get_inputs_from_string(self.agent):
                if input_name not in tools_names:
                    input_names.add(input_name)

        # from the task prompt
        if self.task:
            for input_name in self._get_inputs_from_string(self.task):
                if input_name not in tools_names:
                    input_names.add(input_name)

        # from the tools
        for tool in self.tools:
            if isinstance(tool, Tool) and tool.tool:
                arg_names = [arg.name for arg in tool.arguments]
                for input_name in self._get_inputs_from_string(tool.tool):
                    if input_name not in arg_names and input_name not in tools_names:
                        input_names.add(input_name)

        if not self.task:
            input_names.add("task")

        return {input_name: self.defaults.get(input_name) for input_name in input_names}


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
    actors: dict[str, Actor]

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
