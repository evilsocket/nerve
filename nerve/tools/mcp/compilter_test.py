import unittest
from unittest.mock import MagicMock

from mcp import Tool

from nerve.tools.mcp.client import Client
from nerve.tools.mcp.compiler import create_function_body


class TestCreateFunctionBody(unittest.IsolatedAsyncioTestCase):
    async def test_create_function_body_with_string_argument(self) -> None:
        # Mock the client
        client = MagicMock(spec=Client)

        # Create a mock tool with a simple string argument
        mock_tool = Tool(
            name="hello_world",
            description="A simple hello world function",
            inputSchema={
                "type": "object",
                "properties": {"name": {"type": "string", "description": "The name to greet"}},
                "required": ["name"],
            },
        )

        func_body, type_defs = await create_function_body(client, mock_tool)

        self.assertIn(
            '''
async def hello_world(name: Annotated[str, "The name to greet"]) -> Any:
    """A simple hello world function"""
'''.strip(),
            func_body,
        )

    async def test_create_function_body_with_string_and_int_arguments(self) -> None:
        # Mock the client
        client = MagicMock(spec=Client)

        # Create a mock tool with string and int arguments
        mock_tool = Tool(
            name="calculate",
            description="A function that performs a calculation",
            inputSchema={
                "type": "object",
                "properties": {
                    "operation": {"type": "string", "description": "The operation to perform"},
                    "value": {"type": "integer", "description": "The value to calculate with"},
                },
                "required": ["operation", "value"],
            },
        )

        func_body, type_defs = await create_function_body(client, mock_tool)

        self.assertIn(
            '''
async def calculate(operation: Annotated[str, "The operation to perform"], value: Annotated[int, "The value to calculate with"]) -> Any:
    """A function that performs a calculation"""
'''.strip(),
            func_body,
        )

    async def test_create_function_body_with_string_and_int_arguments_with_default(self) -> None:
        # Mock the client
        client = MagicMock(spec=Client)

        # Create a mock tool with string and int arguments
        mock_tool = Tool(
            name="calculate",
            description="A function that performs a calculation",
            inputSchema={
                "type": "object",
                "properties": {
                    "operation": {"type": "string", "description": "The operation to perform"},
                    "value": {"type": "integer", "description": "The value to calculate with", "default": 10},
                },
                "required": ["operation"],
            },
        )

        func_body, type_defs = await create_function_body(client, mock_tool)

        self.assertIn(
            '''
async def calculate(operation: Annotated[str, "The operation to perform"], value: Annotated[int, "The value to calculate with"] = 10) -> Any:
    """A function that performs a calculation"""
'''.strip(),
            func_body,
        )

    async def test_create_function_body_with_complex_nested_arguments(self) -> None:
        # Mock the client
        client = MagicMock(spec=Client)

        # Create a mock tool with complex nested arguments
        mock_tool = Tool(
            name="process_data",
            description="A function that processes complex data structures",
            inputSchema={
                "type": "object",
                "properties": {
                    "user": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string", "description": "User's name"},
                            "age": {"type": "integer", "description": "User's age"},
                            "preferences": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "User's preferences",
                            },
                        },
                        "description": "User information",
                    },
                    "settings": {
                        "type": "object",
                        "properties": {
                            "enabled": {"type": "boolean", "description": "Whether processing is enabled"},
                            "options": {
                                "type": "object",
                                "properties": {
                                    "mode": {"type": "string", "description": "Processing mode"},
                                    "priority": {"type": "integer", "description": "Processing priority"},
                                },
                                "description": "Processing options",
                            },
                        },
                        "description": "Processing settings",
                    },
                },
                "required": ["user", "settings"],
            },
        )

        func_body, type_defs = await create_function_body(client, mock_tool)

        print()
        print(func_body)
        print()

        self.assertIn(
            '''
async def process_data(user: Annotated[user_0, "User information"], settings: Annotated[settings_1, "Processing settings"]) -> Any:
    """A function that processes complex data structures"""
'''.strip(),
            func_body,
        )
