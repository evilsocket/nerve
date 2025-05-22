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


def wrap_tool_function(func: t.Callable[..., t.Any], mime: str | None = None, paginate: bool = False, max_output: int = 4096) -> t.Callable[..., t.Any]:
    """
    Creates a wrapper around a function that logs the function call and its result.
    If paginate is True, truncates output with a hint if needed.
    """
    async def wrapper(*args: t.Any, **kwargs: t.Any) -> t.Any:
        logger.debug(f"calling {func.__name__} ...")

        state.on_before_tool_called(func.__name__, kwargs)

        started_at = time.time()
        error = None
        try:
            # Get page as string first, convert to integer
            page_str = kwargs.get('page', '1') if paginate else '1'
            try:
                page = int(page_str)
            except (ValueError, TypeError):
                return f"[Error: Invalid page number: {page_str}. Page must be an integer.]"

            # Ensure page is at least 1
            if page < 1:
                 # Decide whether to default to 1 or return an error
                 # For now, let's return an error as per requirement 8
                 return f"[Error: Page number must be a positive integer. Requested page was {page_str}.]"

            result = func(*args, **kwargs)
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
        # Pagination logic
        elif paginate and isinstance(result, str):
            content_bytes = result.encode('utf-8')
            total_size = len(content_bytes)
            page_size = max_output
            total_pages = (total_size + page_size - 1) // page_size

            # Check page again after calculating total_pages for out-of-range error
            if page > total_pages and total_pages > 0: # Only check upper bound if there's at least one page
                 return f"[Error: Requested page {page} is out of range. There are {total_pages} pages.]"

            start = (page - 1) * page_size
            end = min(start + page_size, total_size)
            page_bytes = content_bytes[start:end]
            try:
                page_content = page_bytes.decode('utf-8')
            except UnicodeDecodeError:
                for i in range(len(page_bytes) - 1, -1, -1):
                    try:
                        page_content = page_bytes[:i].decode('utf-8')
                        break
                    except UnicodeDecodeError:
                        continue
                else:
                    page_content = ''
            if total_size > max_output:
                hint = f"[Output truncated. Use the optional 'page' parameter to fetch more. This is page {page} of {total_pages} ({total_size} bytes total).]"
                return page_content + '\n' + hint
            else:
                return page_content
        return result
    wrapper = functools.wraps(func)(wrapper)
    return wrapper


def get_tools_from_namespace(namespace: str, jail: list[str], paginate: bool = False, max_output: int = 4096) -> list[t.Callable[..., t.Any]]:
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
            wrap_tool_function(func, paginate=paginate, max_output=max_output)
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
    paginate: bool = False,
    max_output: int = 4096,
) -> list[t.Callable[..., t.Any]]:
    tools = []

    for namespace in namespaces:
        tools.extend(get_tools_from_namespace(namespace, jail.get(namespace, []), paginate=paginate, max_output=max_output))

    return tools


def get_tools_from_file(working_dir: pathlib.Path, file: str, paginate: bool = False, max_output: int = 4096) -> list[t.Callable[..., t.Any]]:
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
        wrap_tool_function(func, paginate=paginate, max_output=max_output)
        for (name, func) in inspect.getmembers(module, inspect.isfunction)
        if name[0] != "_" and func.__module__ == module.__name__
    ]

    if module_tools:
        logger.debug(f"importing {len(module_tools)} tools from {tool_path}")

    return module_tools


def get_tools_from_files(working_dir: pathlib.Path, files: list[str], paginate: bool = False, max_output: int = 4096) -> list[t.Callable[..., t.Any]]:
    tools = []

    for file in files:
        tools.extend(get_tools_from_file(working_dir, file, paginate=paginate, max_output=max_output))

    return tools


def get_tool_from_yml(working_dir: pathlib.Path, tool: Tool, paginate: bool = False, max_output: int = 4096) -> t.Callable[..., t.Any]:
    # load the template from the same directory as this script
    template_path = os.path.join(os.path.dirname(__file__), "body.j2")
    with open(template_path) as f:
        template_content = f.read()

    logger.debug(f"get_tool_from_yml: tool={tool.name}, paginate={paginate}, max_output={max_output}")

    func_body = (
        jinja2.Environment().from_string(template_content).render(tool=tool, working_dir=str(working_dir.absolute()), paginate=paginate)
    )

    logger.debug(f"compiling tool {tool.name} as:\n{func_body}")

    func_namespace: dict[str, t.Any] = {}
    exec(func_body, func_namespace)

    return wrap_tool_function(func_namespace[tool.name], tool.mime, paginate=paginate, max_output=max_output)


def get_tools_from_yml(working_dir: pathlib.Path, yml_tools: list[Tool], paginate: bool = False, max_output: int = 4096) -> list[t.Callable[..., t.Any]]:
    tools = []

    for tool in yml_tools:
        tools.append(get_tool_from_yml(working_dir, tool, paginate=paginate, max_output=max_output))

    if tools:
        logger.debug(f"importing {len(tools)} tools from {working_dir}")

    return tools
