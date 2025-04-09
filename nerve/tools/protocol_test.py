import unittest
from typing import Annotated

from nerve.tools.protocol import get_tool_schema


class TestProtocol(unittest.IsolatedAsyncioTestCase):
    async def test_function_schema_creation(self) -> None:
        def _test_function(
            first: Annotated[str, "The first argument"],
            second: Annotated[int, "The second argument"] = 0,
        ) -> str | bytes:
            """Description."""
            return "hello"

        schema = get_tool_schema("", _test_function)
        self.assertEqual(
            schema,
            {
                "type": "function",
                "function": {
                    "name": "_test_function",
                    "description": "Description.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "first": {"type": "string", "description": "The first argument"},
                            "second": {"type": "integer", "description": "The second argument"},
                        },
                        "required": ["first"],
                    },
                },
            },
        )
