import pathlib
import unittest
from unittest.mock import MagicMock, patch

import mcp.types as mcp_types

from nerve.models import Configuration
from nerve.runtime import Runtime
from nerve.runtime.runner import Arguments
from nerve.server.mcp import create_mcp_server


class TestMCPServer(unittest.IsolatedAsyncioTestCase):
    @patch("nerve.server.mcp.Runner")
    async def test_mcp_create_server_for_agent_adds_agent_as_tool(self, mock_runner: MagicMock) -> None:
        # Setup
        agent_name = "test_agent"
        config = MagicMock(spec=Configuration)
        config.description = "Test agent description"
        inputs = {"input1": None, "input2": "default_value"}

        run_args = Arguments(
            input_path=pathlib.Path("/path/to/agent"),
            generator="gpt-4",
            conversation_strategy="window",
            conversation_strategy_string="window",
            max_steps=10,
            max_cost=5.0,
            timeout=30,
            quiet=False,
            debug=False,
            log_path=None,
            litellm_debug=False,
            litellm_tracing=None,
            start_state={},
            trace=None,
            task=None,
            interactive=False,
        )

        runtime = MagicMock(spec=Runtime)
        runtime.tools = []

        # Create server
        server = create_mcp_server(
            agent_name=agent_name,
            config=config,
            run_args=run_args,
            inputs=inputs,
            runtime=runtime,
            serve_tools=False,
            tools_only=False,
        )

        list_tools_handler = server.request_handlers[mcp_types.ListToolsRequest]
        server_result = await list_tools_handler(MagicMock())

        self.assertIsInstance(server_result, mcp_types.ServerResult)
        tools_result = server_result.root
        self.assertIsInstance(tools_result, mcp_types.ListToolsResult)

        tools = tools_result.tools  # type: ignore
        self.assertEqual(len(tools), 1)

        tool = tools[0]
        self.assertEqual(tool.name, agent_name)
        self.assertEqual(tool.description, config.description)

        # Verify input schema
        self.assertEqual(tool.inputSchema["type"], "object")
        self.assertEqual(tool.inputSchema["description"], config.description)
        self.assertEqual(tool.inputSchema["required"], ["input1"])  # Only input1 is required
        self.assertIn("input1", tool.inputSchema["properties"])
        self.assertIn("input2", tool.inputSchema["properties"])

    @patch("nerve.server.mcp.Runner")
    async def test_create_mcp_server_for_agent_uses_task_as_description_when_description_not_set(
        self, mock_runner: MagicMock
    ) -> None:
        # Setup
        agent_name = "test_agent"
        config = MagicMock(spec=Configuration)
        config.description = None  # Description not set
        config.task = "Test agent task"  # Task is set
        inputs = {"input1": None, "input2": "default_value"}

        run_args = Arguments(
            input_path=pathlib.Path("/path/to/agent"),
            generator="gpt-4",
            conversation_strategy="window",
            conversation_strategy_string="window",
            max_steps=10,
            max_cost=5.0,
            timeout=30,
            quiet=False,
            debug=False,
            log_path=None,
            litellm_debug=False,
            litellm_tracing=None,
            start_state={},
            trace=None,
            task=None,
            interactive=False,
        )

        runtime = MagicMock(spec=Runtime)
        runtime.tools = []

        # Create server
        server = create_mcp_server(
            agent_name=agent_name,
            config=config,
            run_args=run_args,
            inputs=inputs,
            runtime=runtime,
            serve_tools=False,
            tools_only=False,
        )

        list_tools_handler = server.request_handlers[mcp_types.ListToolsRequest]
        server_result = await list_tools_handler(MagicMock())

        self.assertIsInstance(server_result, mcp_types.ServerResult)
        tools_result = server_result.root
        self.assertIsInstance(tools_result, mcp_types.ListToolsResult)

        tools = tools_result.tools  # type: ignore
        self.assertEqual(len(tools), 1)

        tool = tools[0]
        self.assertEqual(tool.name, agent_name)
        self.assertEqual(tool.description, config.task)  # Description should be the task

        # Verify input schema
        self.assertEqual(tool.inputSchema["type"], "object")
        self.assertEqual(tool.inputSchema["description"], config.task)  # Schema description should also be the task
        self.assertEqual(tool.inputSchema["required"], ["input1"])  # Only input1 is required
        self.assertIn("input1", tool.inputSchema["properties"])
        self.assertIn("input2", tool.inputSchema["properties"])
