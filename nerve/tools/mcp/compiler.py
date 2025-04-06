# ruff: noqa: B010
import os
import pathlib
import typing as t

import jinja2
from mcp import Tool
from pydantic import create_model

from nerve.models import Configuration
from nerve.tools.compiler import wrap_tool_function
from nerve.tools.mcp.client import Client


def _stringify_type(tp: t.Any) -> str:
    origin = t.get_origin(tp)
    args = t.get_args(tp)

    if origin is None:
        return tp.__name__ if hasattr(tp, "__name__") else str(tp)
    else:
        origin_name = origin.__name__ if hasattr(origin, "__name__") else str(origin)
        args_str = ", ".join(_stringify_type(arg) for arg in args)
        return f"{origin_name}[{args_str}]"


def _get_python_type(
    name: str, schema_item: dict[str, t.Any], dyn_defs: dict[str, t.Any] | None = None
) -> tuple[dict[str, t.Any] | None, t.Any]:
    if dyn_defs is None:
        dyn_defs = {}

    type_mapping = {
        "string": str,
        "integer": int,
        "number": float,
        "boolean": bool,
        "array": list,
        "object": dict,
        "null": None,
    }

    schema_type = schema_item.get("type")

    if schema_type == "array":
        items = schema_item.get("items", {})
        sub_dyn_defs, item_type = _get_python_type(name, items, dyn_defs)
        if sub_dyn_defs:
            dyn_defs.update(sub_dyn_defs)

        return dyn_defs, list[item_type]  # type: ignore
    elif schema_type == "object":
        properties = schema_item.get("properties", {})
        required = schema_item.get("required", [])

        field_definitions = {}
        for prop_name, prop_schema in properties.items():
            sub_dyn_defs, prop_type = _get_python_type(prop_name, prop_schema, dyn_defs)
            is_required = prop_name in required

            if sub_dyn_defs:
                dyn_defs.update(sub_dyn_defs)

            if is_required:
                field_definitions[prop_name] = (prop_type, ...)
            else:
                field_definitions[prop_name] = (prop_type | None, None)  # type: ignore

        # create a unique name for this model
        dyn_type_name = f"{name}_{len(dyn_defs)}"
        dyn_type = create_model(dyn_type_name, **field_definitions)  # type: ignore
        dyn_type.__module__ = None

        # store description metadata on the model if available
        for prop_name, prop_schema in properties.items():
            if "description" in prop_schema:
                setattr(dyn_type.__fields__[prop_name], "description", prop_schema["description"])

        dyn_defs[dyn_type.__name__] = dyn_type

        return dyn_defs, dyn_type
    else:
        return None, type_mapping.get(schema_type, t.Any)  # type: ignore


async def create_function_body(client: Client, mcp_tool: Tool) -> tuple[str, dict[str, t.Any]]:
    typed_args = []
    client_name = f"nerve_mcp_{client.name}_client"
    type_defs = {client_name: client}

    for name, arg_props in mcp_tool.inputSchema.get("properties", {}).items():
        # print(name, arg_props)
        args_def, arg_type = _get_python_type(name, arg_props)
        arg = {"name": name, "type": _stringify_type(arg_type), "description": arg_props.get("description", "")}

        if args_def:
            type_defs.update(args_def)

        if "default" in arg_props:
            arg["default"] = arg_props["default"]

        typed_args.append(arg)

    # load the template from the same directory as this script
    template_path = os.path.join(os.path.dirname(__file__), "body.j2")
    with open(template_path) as f:
        template_content = f.read()

    return (
        jinja2.Environment()
        .from_string(template_content)
        .render(client_name=client_name, tool=mcp_tool, arguments=typed_args),
        type_defs,
    )


async def get_tools_from_mcp(
    name: str, server: Configuration.MCPServer, working_dir: pathlib.Path
) -> list[t.Callable[..., t.Any]]:
    # connect and list tools
    client = Client(name, server, working_dir)
    mpc_tools = await client.tools()
    compiled_tools = []

    for mcp_tool in mpc_tools:
        func_body, type_defs = await create_function_body(client, mcp_tool)

        # print(func_body)
        exec(func_body, type_defs)

        tool_fn = wrap_tool_function(type_defs[mcp_tool.name])
        compiled_tools.append(tool_fn)
    return compiled_tools
