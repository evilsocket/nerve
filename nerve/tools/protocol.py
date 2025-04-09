import inspect
import typing as t
from typing import Annotated

from loguru import logger


def get_tool_schema(generator: str, func: t.Callable[..., t.Any]) -> dict[str, t.Any]:
    signature = inspect.signature(func)
    docstring = inspect.getdoc(func) or ""

    logger.debug("signature: {}", signature)

    if docstring == "":
        logger.debug(f"tool {func.__name__} has no docstring")

    is_gemini = "gemini" in generator or "google" in generator
    type_hints = t.get_type_hints(func, include_extras=True)
    tool = {
        "type": "function",
        "function": {
            "name": func.__name__,
            "description": docstring,
            "parameters": {"type": "object", "properties": {}, "required": []},
        },
    }

    for param_name, param in signature.parameters.items():
        if param_name == "self":
            logger.debug("ignoring self parameter")
            continue

        param_type = type_hints.get(param_name)
        if param_type is None:
            logger.debug("parameter {} has no type hint", param_name)
            continue

        is_annotated = t.get_origin(param_type) is Annotated

        if is_annotated:
            args = t.get_args(param_type)
            base_type = args[0]
            description = args[1] if len(args) > 1 else ""
        else:
            base_type = param_type
            description = ""

        param_schema = process_type(base_type)

        logger.debug("{param_name}: {param_schema}", param_name=param_name, param_schema=param_schema)

        if description:
            if isinstance(description, str):
                param_schema["description"] = description
            else:
                if hasattr(description, "description"):
                    param_schema["description"] = description.description

                # gemini will raise an INVALID_ARGUMENT if examples are provided
                if hasattr(description, "examples") and not is_gemini:
                    param_schema["examples"] = description.examples

        tool["function"]["parameters"]["properties"][param_name] = param_schema  # type: ignore

        if param.default is param.empty:
            tool["function"]["parameters"]["required"].append(param_name)  # type: ignore

    if not tool["function"]["parameters"]["properties"] and is_gemini:  # type: ignore
        # gemini will raise an INVALID_ARGUMENT if parameters are empty
        logger.debug("removing empty parameters from tool {}: {}", func.__name__, tool)
        del tool["function"]["parameters"]  # type: ignore

    return tool


def get_tool_response(response: t.Any) -> t.Any:
    response = response or ""
    if isinstance(response, str):
        # simple case, just text
        return response
    elif isinstance(response, bytes):
        # binary data, attempt decoding
        try:
            return response.decode("utf-8", errors="ignore")
        except UnicodeDecodeError:
            return str(response)

    elif isinstance(response, dict):
        # structured (vision), return as list
        return response
    elif isinstance(response, list):
        # list of responses, return as list
        return [get_tool_response(r) for r in response]
    else:
        logger.warning(f"unknown tool response type: {type(response)}")
        return get_tool_response(str(response))


def process_type(type_annotation: t.Any) -> dict[str, t.Any]:
    if type_annotation is str:
        return {"type": "string"}
    elif type_annotation is int:
        return {"type": "integer"}
    elif type_annotation is float:
        return {"type": "number"}
    elif type_annotation is bool:
        return {"type": "boolean"}
    elif type_annotation is list or t.get_origin(type_annotation) is list:
        item_type = t.get_args(type_annotation)[0] if t.get_args(type_annotation) else t.Any
        return {"type": "array", "items": process_type(item_type)}
    elif type_annotation is dict or t.get_origin(type_annotation) is dict:
        return {"type": "object"}

    if isinstance(type_annotation, type):
        return process_typed_dict(type_annotation)

    return {"type": "object"}


def process_typed_dict(typed_dict_class: type) -> dict[str, t.Any]:
    annotations = t.get_type_hints(typed_dict_class, include_extras=True)

    schema = {
        "type": "object",
        "properties": {},
        "required": [],
    }

    for field_name, field_type in annotations.items():
        is_annotated = t.get_origin(field_type) is Annotated

        if is_annotated:
            args = t.get_args(field_type)
            base_type = args[0]
            field_schema = process_type(base_type)

            for arg in args[1:]:
                if hasattr(arg, "description") and arg.description:
                    field_schema["description"] = arg.description
                if hasattr(arg, "examples") and arg.examples:
                    field_schema["examples"] = arg.examples
        else:
            field_schema = process_type(field_type)

        schema["properties"][field_name] = field_schema  # type: ignore

    return schema
