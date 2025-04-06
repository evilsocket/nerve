import asyncio
import base64
import functools
import importlib
import inspect
import os
import pathlib
import time
import traceback
import typing as t

import jinja2
from loguru import logger
from termcolor import colored

from nerve.models import Tool
from nerve.runtime import state


def wrap_tool_function(func: t.Callable[..., t.Any], mime: str | None = None) -> t.Callable[..., t.Any]:
    """
    Creates a wrapper around a function that logs the function call and its result.
    """

    async def wrapper(*args: t.Any, **kwargs: t.Any) -> t.Any:
        logger.debug(f"calling {func.__name__} ...")

        state.on_before_tool_called(func.__name__, kwargs)

        started_at = time.time()
        error = None
        try:
            result = func(*args, **kwargs)
            # check if the tool function returned a coroutine
            if asyncio.iscoroutine(result):
                result = await result

            logger.debug(f"tool {func.__name__} returned: {result}")

        except Exception as e:
            result = f"ERROR in {func.__name__}: {e}"
            error = str(e)
            logger.error(colored(f"{func.__name__}: {e}", "red"))
            logger.debug(traceback.format_exc())

        finished_at = time.time()

        state.on_tool_called(started_at, finished_at, func.__name__, kwargs, result, error)

        if mime:
            if mime.startswith("image/"):
                result = {
                    "type": "image_url",
                    "image_url": {"url": f"data:{mime};base64,{base64.b64encode(result).decode('utf-8')}"},
                }
            else:
                logger.error(f"tool {func.__name__} references an unsupported mime type: {mime}")
                exit(1)

        return result

    # Preserve the function's metadata
    wrapper = functools.wraps(func)(wrapper)

    return wrapper


def get_tools_from_namespace(namespace: str, jail: list[str]) -> list[t.Callable[..., t.Any]]:
    try:
        importlib.util.find_spec(f"nerve.tools.namespaces.{namespace}")
    except ImportError as err:
        raise ImportError(f"namespace {namespace} not found") from err

    try:
        module = __import__(f"nerve.tools.namespaces.{namespace}", fromlist=[""])
        if jail:
            for jailed_path in jail:
                module.jail.append(jailed_path)
                logger.debug(f"namespace {namespace} jailed to: {jailed_path}")

        module_tools = [
            wrap_tool_function(func)
            for (name, func) in inspect.getmembers(module, inspect.isfunction)
            if name[0] != "_" and func.__module__ == module.__name__
        ]
        logger.debug(f"importing {len(module_tools)} tools from {namespace} namespace")
        return module_tools
    except ImportError as err:
        raise ImportError(f"could not import {namespace}: {err}") from err


def get_tools_from_namespaces(
    namespaces: list[str],
    jail: dict[str, list[str]],
) -> list[t.Callable[..., t.Any]]:
    tools = []

    for namespace in namespaces:
        tools.extend(get_tools_from_namespace(namespace, jail.get(namespace, [])))

    return tools


def get_tools_from_file(working_dir: pathlib.Path, file: str) -> list[t.Callable[..., t.Any]]:
    tool_path = pathlib.Path(file)
    if not tool_path.is_absolute():
        tool_path = working_dir / tool_path

    if not tool_path.exists():
        tool_path = tool_path.with_suffix(".py")
        if not tool_path.exists():
            raise FileNotFoundError(f"tool {file} does not exist")

    spec = importlib.util.spec_from_file_location(tool_path.stem, tool_path)
    module = importlib.util.module_from_spec(spec)  # type: ignore
    spec.loader.exec_module(module)  # type: ignore

    module_tools = [
        wrap_tool_function(func)
        for (name, func) in inspect.getmembers(module, inspect.isfunction)
        if name[0] != "_" and func.__module__ == module.__name__
    ]

    if module_tools:
        logger.debug(f"importing {len(module_tools)} tools from {tool_path}")

    return module_tools


def get_tools_from_files(working_dir: pathlib.Path, files: list[str]) -> list[t.Callable[..., t.Any]]:
    tools = []

    for file in files:
        tools.extend(get_tools_from_file(working_dir, file))

    return tools


def get_tool_from_yml(working_dir: pathlib.Path, tool: Tool) -> t.Callable[..., t.Any]:
    # load the template from the same directory as this script
    template_path = os.path.join(os.path.dirname(__file__), "body.j2")
    with open(template_path) as f:
        template_content = f.read()

    func_body = (
        jinja2.Environment().from_string(template_content).render(tool=tool, working_dir=str(working_dir.absolute()))
    )

    logger.debug(f"compiling tool {tool.name} as:\n{func_body}")

    func_namespace: dict[str, t.Any] = {}
    exec(func_body, func_namespace)

    return wrap_tool_function(func_namespace[tool.name], tool.mime)


def get_tools_from_yml(working_dir: pathlib.Path, yml_tools: list[Tool]) -> list[t.Callable[..., t.Any]]:
    tools = []

    for tool in yml_tools:
        tools.append(get_tool_from_yml(working_dir, tool))

    if tools:
        logger.debug(f"importing {len(tools)} tools from {working_dir}")

    return tools
