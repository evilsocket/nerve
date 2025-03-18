import pathlib
import typing as t

from loguru import logger

from nerve.models import Tool
from nerve.tools import compiler


class Runtime:
    def __init__(self, name: str, generator: str, working_dir: pathlib.Path):
        # task unique identifier
        self.name = name
        # which model will handle this task
        self.generator = generator
        # the working directory for the task
        self.working_dir = working_dir
        # full chat history of the task
        self.history: list[t.Any] = []
        # available tools
        self.tools: list[t.Callable[..., t.Any]] = []

    @classmethod
    def build(
        cls,
        working_dir: pathlib.Path,
        name: str,
        generator: str,
        using: list[str],
        jail: dict[str, list[str]],
        tools: list[Tool | t.Callable[..., t.Any]],
    ) -> "Runtime":
        runtime = cls(name=name, generator=generator, working_dir=working_dir)

        # import tools from builtin namespaces
        ns_tools = compiler.get_tools_from_namespaces(using, jail)
        if ns_tools:
            logger.debug(f"🧰 importing {len(ns_tools)} tools from: {using}")
            runtime.tools.extend(ns_tools)

        # import custom tools from yaml definition
        yml_tools = compiler.get_tools_from_yml(
            working_dir,
            [tool for tool in tools if isinstance(tool, Tool) and not tool.path],
        )
        if yml_tools:
            logger.debug(f"🧰 importing {len(yml_tools)} tools from: {working_dir}")
            runtime.tools.extend(yml_tools)

        # import custom tools from files
        py_tools = compiler.get_tools_from_files(
            working_dir,
            [tool.path for tool in tools if isinstance(tool, Tool) and tool.path],
        )
        if py_tools:
            logger.debug(f"🧰 importing {len(py_tools)} tools from: {working_dir}")
            runtime.tools.extend(py_tools)

        # import custom tools from functions (when used as sdk)
        funcs = [compiler.wrap_tool_function(tool) for tool in tools if callable(tool)]
        if funcs:
            logger.debug(f"🧰 importing {len(funcs)} custom tools from functions")
            runtime.tools.extend(funcs)

        return runtime
