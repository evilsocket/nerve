import typing as t

from loguru import logger

from nerve.generation import WindowStrategy


class FullHistoryStrategy(WindowStrategy):
    async def get_window(self, history: list[dict[str, t.Any]]) -> list[dict[str, t.Any]]:
        return history

    def __str__(self) -> str:
        return "<full history>"


class SlidingWindowStrategy(WindowStrategy):
    def __init__(self, window_size: int = 10) -> None:
        self.window_size = window_size

    async def get_window(self, history: list[dict[str, t.Any]]) -> list[dict[str, t.Any]]:
        history_size = len(history)
        if history_size <= self.window_size:
            return history

        for msg in history:
            logger.debug(" * " + str(msg))
        logger.debug("---")

        # NOTE: This is a "best effort" sliding window strategy.
        #
        # In theory we could just get the last N messages, however for each
        # tool call from the AI, we need to add its tool call response or the
        # API errors with:
        #
        #   Invalid parameter: messages with role 'tool' must be a response
        #   to a preceeding message with 'tool_calls'.
        #
        # So we start from there and go backwards until we include everything we need.
        idx_actual = history_size - self.window_size
        logger.debug(f"idx_actual={idx_actual}")
        while idx_actual > 0:
            item = history[idx_actual]

            logger.debug(f"  item={item}")
            # if the item is not a tool response (as dictionary), it means
            # it is a tool call and everything we have after it is a response
            # for it, so we're done.
            if not isinstance(item, dict) or item.get("role") != "tool":
                break
            # keep going backwards until we find a tool call
            idx_actual -= 1

        window = history[idx_actual:]

        logger.debug(f"window_size={self.window_size}, window_size_actual={len(window)}, idx_actual_after={idx_actual}")

        for msg in window:
            logger.debug(" [*] " + str(msg))

        return window

    def __str__(self) -> str:
        return f"<sliding window of size {self.window_size}>"


def strategy_from_string(strategy: str) -> WindowStrategy:
    if strategy == "full":
        return FullHistoryStrategy()
    elif strategy.isdigit():
        return SlidingWindowStrategy(int(strategy))
    else:
        raise ValueError(f"Invalid conversation strategy: {strategy}")
