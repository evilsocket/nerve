import pathlib
import tempfile
import unittest

from nerve.models import Configuration, Tool


class TestConfiguration(unittest.TestCase):
    def test_is_legacy_with_system_prompt(self) -> None:
        """Test that is_legacy returns True when system_prompt is set"""
        config = Configuration(system_prompt="This is a legacy prompt")
        self.assertTrue(config.is_legacy)

    def test_is_legacy_without_system_prompt(self) -> None:
        """Test that is_legacy returns False when system_prompt is None"""
        config = Configuration(agent="Agent prompt", task="Task description")
        self.assertFalse(config.is_legacy)

    def test_is_legacy_with_empty_config(self) -> None:
        """Test that is_legacy returns False with default configuration"""
        config = Configuration()
        self.assertFalse(config.is_legacy)

    def test_system_prompt_excluded_from_serialization(self) -> None:
        """Test that system_prompt is excluded when serializing the configuration"""
        config = Configuration(system_prompt="This should be excluded", agent="Agent prompt")
        serialized = config.model_dump()
        self.assertNotIn("system_prompt", serialized)

    def test_system_prompt_loaded_from_file(self) -> None:
        """Test that system_prompt is loaded if present in a file"""

        with tempfile.NamedTemporaryFile(suffix=".yml", delete=False) as temp_file:
            temp_file.write(b"system_prompt: Legacy prompt")
            temp_path = pathlib.Path(temp_file.name)

        try:
            loaded_config = Configuration.from_path(temp_path)

            self.assertEqual(loaded_config.system_prompt, "Legacy prompt")
            self.assertTrue(loaded_config.is_legacy)
        finally:
            temp_path.unlink()

    def test_get_inputs_from_agent_and_task(self) -> None:
        """Test that get_inputs extracts variables from both agent and task prompts"""
        config = Configuration(
            agent="I am an agent with {{input1}} and {{input2}}",
            task="Complete the task using {{input2}} and {{input3}}",
        )
        inputs = config.get_inputs()

        self.assertIn("input1", inputs)
        self.assertIn("input2", inputs)
        self.assertIn("input3", inputs)
        self.assertIsNone(inputs["input1"])
        self.assertIsNone(inputs["input2"])
        self.assertIsNone(inputs["input3"])

    def test_get_inputs_with_defaults(self) -> None:
        """Test that get_inputs includes default values when available"""
        config = Configuration(
            agent="I am an agent with {{input1}}",
            task="Complete the task with {{input2}}",
            defaults={"input1": "default1", "input2": "default2", "unused": "value"},
        )
        inputs = config.get_inputs()

        self.assertEqual(inputs["input1"], "default1")
        self.assertEqual(inputs["input2"], "default2")
        self.assertNotIn("unused", inputs)

    def test_get_inputs_from_tools(self) -> None:
        """Test that get_inputs extracts variables from tool definitions"""

        config = Configuration(
            agent="I am an agent",
            tools=[
                Tool(
                    name="test_tool",
                    tool="This tool uses {{global_var}} and {{tool_arg}}",
                    arguments=[Tool.Argument(name="tool_arg", description="A tool variable")],
                )
            ],
        )
        inputs = config.get_inputs()

        self.assertIn("global_var", inputs)
        self.assertNotIn("tool_arg", inputs)  # Should be excluded as it's a tool argument

    def test_get_inputs_from_all_sources(self) -> None:
        """Test that get_inputs extracts variables from all sources"""
        config = Configuration(
            agent="I am an agent with {{input1}}",
            task="Complete the task with {{input2}}",
            tools=[
                Tool(
                    name="test_tool",
                    tool="This tool uses {{global_var}} and {{tool_arg}}",
                    arguments=[Tool.Argument(name="tool_arg", description="A tool variable")],
                )
            ],
        )
        inputs = config.get_inputs()

        self.assertIn("input1", inputs)
        self.assertIn("input2", inputs)
        self.assertIn("global_var", inputs)
        self.assertNotIn("tool_arg", inputs)

    def test_get_inputs_includes_task_when_task_not_set(self) -> None:
        """Test that get_inputs includes 'task' when configuration.task is not set"""
        config = Configuration(
            agent="I am an agent with {{input1}}",
            task=None,
            defaults={"input1": "default1"},
        )
        inputs = config.get_inputs()

        self.assertIn("task", inputs)
        self.assertIn("input1", inputs)

    def test_get_inputs_ignores_interpolated_tool_calls(self) -> None:
        """Test that get_inputs doesn't extract variables from interpolated tool calls"""
        config = Configuration(
            agent="I am an agent",
            task="Complete the task using {{tool_name(param='value')}} and {{input1}}",
            tools=[
                Tool(
                    name="tool_name",
                    description="A test tool",
                )
            ],
        )
        inputs = config.get_inputs()

        self.assertIn("input1", inputs)
        self.assertNotIn("tool_name", inputs)  # Should be excluded as it's a tool
        self.assertNotIn("param", inputs)  # Should be excluded as it's a parameter to a tool call
