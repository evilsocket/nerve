import asyncio
import typing as t
import unittest
from unittest.mock import patch

from nerve.generation.conversation import (
    FullHistoryStrategy,
    SlidingWindowStrategy,
    StrippedWindowStrategy,
    strategy_from_string,
)


class TestFullHistoryStrategy(unittest.TestCase):
    def test_get_window_returns_full_history(self) -> None:
        strategy = FullHistoryStrategy()
        history = [{"role": "user", "content": "Hello"}, {"role": "assistant", "content": "Hi"}]

        result = asyncio.run(strategy.get_window(history))

        self.assertEqual(result, history)
        self.assertEqual(str(strategy), "<full history>")


class TestSlidingWindowStrategy(unittest.TestCase):
    def test_get_window_returns_full_history_when_smaller_than_window(self) -> None:
        strategy = SlidingWindowStrategy(window_size=5)
        history = [{"role": "user", "content": "Hello"}, {"role": "assistant", "content": "Hi"}]

        result = asyncio.run(strategy.get_window(history))

        self.assertEqual(result, history)

    def test_get_window_returns_window_sized_history(self) -> None:
        strategy = SlidingWindowStrategy(window_size=2)
        history = [
            {"role": "user", "content": "First"},
            {"role": "user", "content": "Second"},
            {"role": "user", "content": "Third"},
            {"role": "user", "content": "Fourth"},
        ]

        result = asyncio.run(strategy.get_window(history))

        self.assertEqual(len(result), 2)
        self.assertEqual(result, history[2:])

    @patch("nerve.generation.conversation.logger.debug")
    def test_get_window_logs_messages(self, mock_debug: unittest.mock.Mock) -> None:
        strategy = SlidingWindowStrategy(window_size=2)
        history = [
            {"role": "user", "content": "First"},
            {"role": "assistant", "content": "First response"},
            {"role": "user", "content": "Second"},
        ]

        asyncio.run(strategy.get_window(history))

        # Verify logger was called
        self.assertTrue(mock_debug.called)

    def test_str_representation(self) -> None:
        strategy = SlidingWindowStrategy(window_size=10)
        self.assertEqual(str(strategy), "<sliding window of size 10>")


class TestStrippedWindowStrategy(unittest.TestCase):
    def test_get_window_returns_full_history_when_smaller_than_window(self) -> None:
        strategy = StrippedWindowStrategy(window_size=5)
        history = [{"role": "user", "content": "Hello"}, {"role": "assistant", "content": "Hi"}]

        result = asyncio.run(strategy.get_window(history))

        self.assertEqual(result, history)

    def test_get_window_strips_content_outside_window(self) -> None:
        strategy = StrippedWindowStrategy(window_size=2)
        history = [
            {"role": "user", "content": "First"},
            {"role": "user", "content": "Second"},
            {"role": "user", "content": "Third"},
            {"role": "user", "content": "Fourth"},
        ]

        result = asyncio.run(strategy.get_window(history))

        # All messages should be present
        self.assertEqual(len(result), 4)

        # First two messages should be stripped
        self.assertEqual(result[0]["content"], "<stripped content>")
        self.assertEqual(result[1]["content"], "<stripped content>")

        # Last two messages should be preserved
        self.assertEqual(result[2]["content"], "Third")
        self.assertEqual(result[3]["content"], "Fourth")

    def test_get_window_strips_tool_messages(self) -> None:
        strategy = StrippedWindowStrategy(window_size=1)
        history = [
            {"role": "user", "content": "First"},
            {"role": "tool", "tool_call_id": "call_123", "content": "Tool response"},
            {"role": "user", "content": "Third"},
        ]

        result = asyncio.run(strategy.get_window(history))

        # First message should be stripped
        self.assertEqual(result[0]["content"], "<stripped content>")

        # Tool message should be stripped but preserve tool_call_id
        self.assertEqual(result[1]["content"], "<stripped tool response>")
        self.assertEqual(result[1]["tool_call_id"], "call_123")

        # Last message should be preserved
        self.assertEqual(result[2]["content"], "Third")

    def test_str_representation(self) -> None:
        strategy = StrippedWindowStrategy(window_size=10)
        self.assertEqual(str(strategy), "<stripping window of size 10>")

    def test_strip_strategy_from_string(self) -> None:
        strategy = strategy_from_string("strip-8")
        self.assertIsInstance(strategy, StrippedWindowStrategy)
        self.assertEqual(t.cast(StrippedWindowStrategy, strategy).window_size, 8)


class TestStrategyFromString(unittest.TestCase):
    def test_full_strategy(self) -> None:
        strategy = strategy_from_string("full")
        self.assertIsInstance(strategy, FullHistoryStrategy)

    def test_sliding_window_strategy(self) -> None:
        strategy = strategy_from_string("5")
        self.assertIsInstance(strategy, SlidingWindowStrategy)
        self.assertEqual(t.cast(SlidingWindowStrategy, strategy).window_size, 5)

    def test_stripped_window_strategy(self) -> None:
        strategy = strategy_from_string("strip-2")
        self.assertIsInstance(strategy, StrippedWindowStrategy)
        self.assertEqual(t.cast(StrippedWindowStrategy, strategy).window_size, 2)

    def test_invalid_strategy(self) -> None:
        with self.assertRaises(ValueError):
            strategy_from_string("invalid")
