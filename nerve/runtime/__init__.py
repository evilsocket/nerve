import pathlib
import typing as t

from loguru import logger

from nerve.models import Configuration, Tool
from nerve.tools import compiler
from nerve.tools.mcp import compiler as mcp_compiler


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
    async def build(
        cls,
        working_dir: pathlib.Path,
        name: str,
        configuration: Configuration,
    ) -> "Runtime":
        runtime = cls(name=name, generator=configuration.generator or "", working_dir=working_dir)

        # import tools from builtin namespaces
        ns_tools = compiler.get_tools_from_namespaces(configuration.using, configuration.jail)
        if ns_tools:
            logger.debug(f"🧰 importing {len(ns_tools)} tools from: {configuration.using}")
            runtime.tools.extend(ns_tools)

        # import custom tools from yaml definition
        yml_tools = compiler.get_tools_from_yml(
            working_dir,
            [tool for tool in configuration.tools if isinstance(tool, Tool) and not tool.path],
        )
        if yml_tools:
            logger.debug(f"🧰 importing {len(yml_tools)} tools from: {working_dir}")
            runtime.tools.extend(yml_tools)

        # import custom tools from files
        py_tools = compiler.get_tools_from_files(
            working_dir,
            [tool.path for tool in configuration.tools if isinstance(tool, Tool) and tool.path],
        )
        if py_tools:
            logger.debug(f"🧰 importing {len(py_tools)} tools from: {working_dir}")
            runtime.tools.extend(py_tools)

        # import custom tools from functions (when used as sdk)
        funcs = [compiler.wrap_tool_function(tool) for tool in configuration.tools if callable(tool)]
        if funcs:
            logger.debug(f"🧰 importing {len(funcs)} custom tools")
            runtime.tools.extend(funcs)

        # import MCP servers
        for name, server in configuration.mcp.items():
            server_tools = await mcp_compiler.get_tools_from_mcp(name, server)
            logger.info(f"🧰 importing {len(server_tools)} tools from MCP server {name}")
            runtime.tools.extend(server_tools)

        return runtime
