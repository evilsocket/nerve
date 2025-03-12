import os
from unittest.mock import MagicMock, patch

import pytest

from nerve.models import Mode
from nerve.runtime import state


class TestInterpolate:
    def setup_method(self) -> None:
        # Reset state before each test
        state.reset()
        state._variables = {}
        state._defaults = {}
        state._mode = Mode.AUTOMATIC
        state._tools = {}

    def test_basic_interpolation(self) -> None:
        # Setup
        state.update_variables({"name": "John", "age": 30})

        # Test
        result = state.interpolate("Hello, {{ name }}! You are {{ age }} years old.")

        # Verify
        assert result == "Hello, John! You are 30 years old."

    def test_interpolation_with_extra_context(self) -> None:
        # Setup
        state.update_variables({"name": "John"})
        extra = {"location": "New York"}

        # Test
        result = state.interpolate("{{ name }} is from {{ location }}.", extra)

        # Verify
        assert result == "John is from New York."

    def test_undefined_variable_in_automatic_mode(self) -> None:
        # Setup
        state._mode = Mode.AUTOMATIC
        state._defaults = {"city": "San Francisco"}

        # Test
        result = state.interpolate("Welcome to {{ city }}!")

        # Verify
        assert result == "Welcome to San Francisco!"

    def test_undefined_variable_from_environment(self) -> None:
        # Setup
        with patch.dict(os.environ, {"TEST_VAR": "test_value"}):
            # Test
            result = state.interpolate("Environment value: {{ TEST_VAR }}")

            # Verify
            assert result == "Environment value: test_value"
            # The variable should also be saved to state
            assert state._variables.get("TEST_VAR") == "test_value"

    @patch("builtins.input", return_value="Paris")
    def test_undefined_variable_in_interactive_mode(self, mock_input: MagicMock) -> None:
        # Setup
        state._mode = Mode.INTERACTIVE

        # Test
        result = state.interpolate("Welcome to {{ city }}!")

        # Verify
        assert result == "Welcome to Paris!"
        assert mock_input.called
        assert state._variables.get("city") == "Paris"

    def test_tool_call_in_template(self) -> None:
        # Setup
        mock_tool = MagicMock(return_value="tool result")
        state._tools = {"test_tool": mock_tool}

        # Test
        result = state.interpolate("Tool output: {{ test_tool() }}")

        # Verify
        assert result == "Tool output: tool result"
        assert mock_tool.called

    @patch("nerve.runtime.state.on_user_input_needed")
    def test_missing_variable_raises_exception(self, mock_input_needed: MagicMock) -> None:
        # Setup
        state._mode = Mode.AUTOMATIC
        mock_input_needed.side_effect = Exception("Missing parameter")

        # Test & Verify
        with pytest.raises(Exception):  # noqa: B017
            state.interpolate("Missing {{ variable }}")

    def test_complex_expression(self) -> None:
        # Setup
        state.update_variables({"numbers": [1, 2, 3, 4, 5]})

        # Test
        result = state.interpolate("Sum: {{ numbers|sum }}, Average: {{ (numbers|sum) / numbers|length }}")

        # Verify
        assert result == "Sum: 15, Average: 3.0"
