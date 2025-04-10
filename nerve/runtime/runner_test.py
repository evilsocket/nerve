import unittest

from nerve.runtime.runner import _parse_events


class TestParseEvents(unittest.TestCase):
    def test_task_completed(self) -> None:
        """Test when task is completed successfully."""
        inputs = {"input1": "value1"}
        events = [
            {"name": "flow_start", "timestamp": 0.0},
            {"name": "task_complete", "timestamp": 1.0, "data": {"output": "result"}},
            {"name": "flow_complete", "timestamp": 2.0, "data": {"steps": 3, "usage": {"tokens": 100}}},
        ]

        result = _parse_events(inputs, events)

        self.assertTrue(result.task_success)
        self.assertEqual(result.output_object, {"output": "result"})
        self.assertEqual(result.steps, 3)
        self.assertEqual(result.time, 2.0)
        self.assertEqual(result.usage, {"tokens": 100})

    def test_task_completed_with_reason(self) -> None:
        """Test when task is completed with a reason."""
        inputs = {"input1": "value1"}
        events = [
            {"name": "flow_start", "timestamp": 0.0},
            {"name": "task_complete", "timestamp": 1.0, "data": {"reason": "Task finished successfully"}},
            {"name": "flow_complete", "timestamp": 2.0, "data": {"steps": 2, "usage": {"tokens": 50}}},
        ]

        result = _parse_events(inputs, events)

        self.assertTrue(result.task_success)
        self.assertEqual(result.output_object, {"reason": "Task finished successfully"})
        self.assertEqual(result.steps, 2)
        self.assertEqual(result.time, 2.0)
        self.assertEqual(result.usage, {"tokens": 50})

    def test_task_failed(self) -> None:
        """Test when task fails."""
        inputs = {"input1": "value1"}
        events = [
            {"name": "flow_start", "timestamp": 0.0},
            {"name": "task_failed", "timestamp": 1.0, "data": {"error": "Something went wrong"}},
            {"name": "flow_complete", "timestamp": 2.0, "data": {"steps": 1, "usage": {"tokens": 20}}},
        ]

        result = _parse_events(inputs, events)

        self.assertFalse(result.task_success)
        self.assertEqual(result.output_object, {"error": "Something went wrong"})
        self.assertEqual(result.steps, 1)
        self.assertEqual(result.time, 2.0)
        self.assertEqual(result.usage, {"tokens": 20})

    def test_task_failed_with_reason(self) -> None:
        """Test when task fails with a reason."""
        inputs = {"input1": "value1"}
        events = [
            {"name": "flow_start", "timestamp": 0.0},
            {"name": "task_failed", "timestamp": 1.0, "data": {"reason": "Invalid input"}},
            {"name": "flow_complete", "timestamp": 2.0, "data": {"steps": 1, "usage": {"tokens": 20}}},
        ]

        result = _parse_events(inputs, events)

        self.assertFalse(result.task_success)
        self.assertEqual(result.output_object, {"reason": "Invalid input"})
        self.assertEqual(result.steps, 1)
        self.assertEqual(result.time, 2.0)
        self.assertEqual(result.usage, {"tokens": 20})

    def test_flow_completed_with_variables(self) -> None:
        """Test when flow completes with variables in state."""
        inputs = {"input1": "value1"}
        events = [
            {"name": "flow_start", "timestamp": 0.0},
            {
                "name": "flow_complete",
                "timestamp": 2.0,
                "data": {
                    "steps": 4,
                    "usage": {"tokens": 150},
                    "state": {"variables": {"input1": "value1", "output1": "result1"}},
                },
            },
        ]

        result = _parse_events(inputs, events)

        self.assertEqual(result.output_object, {"output1": "result1"})
        self.assertEqual(result.steps, 4)
        self.assertEqual(result.time, 2.0)
        self.assertEqual(result.usage, {"tokens": 150})

    def test_fallback_to_text_response(self) -> None:
        """Test fallback to text response when no task completion events."""
        inputs = {"input1": "value1"}
        events = [
            {"name": "flow_start", "timestamp": 0.0},
            {"name": "text_response", "timestamp": 1.0, "data": {"response": "This is a text response"}},
        ]

        result = _parse_events(inputs, events)

        self.assertEqual(result.output_object, {"response": "This is a text response"})
        self.assertEqual(result.steps, 0)
        self.assertEqual(result.time, 0.0)
        self.assertEqual(result.usage, {})

    def test_fallback_to_tool_called(self) -> None:
        """Test fallback to tool called when no task completion or text response."""
        inputs = {"input1": "value1"}
        events = [
            {"name": "flow_start", "timestamp": 0.0},
            {"name": "tool_called", "timestamp": 1.0, "data": {"result": "Tool execution result"}},
        ]

        result = _parse_events(inputs, events)

        self.assertEqual(result.output_object, {"output": "Tool execution result"})
        self.assertEqual(result.steps, 0)
        self.assertEqual(result.time, 0)
        self.assertEqual(result.usage, {})

    def test_no_relevant_events(self) -> None:
        """Test when no relevant events are found."""
        inputs = {"input1": "value1"}
        events = [
            {"name": "flow_start", "timestamp": 0.0},
        ]

        result = _parse_events(inputs, events)

        self.assertIsNone(result.output_object)
        self.assertFalse(result.task_success)
        self.assertEqual(result.steps, 0)
        self.assertEqual(result.time, 0.0)
        self.assertEqual(result.usage, {})
