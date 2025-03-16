import typing as t

from loguru import logger

from nerve.generation import WindowStrategy


class FullHistoryStrategy(WindowStrategy):
    async def get_window(self, history: list[dict[str, t.Any]]) -> list[dict[str, t.Any]]:
        return history

    def __str__(self) -> str:
        return "<full history>"


# TODO: add a Compression strategy that leaves the conversation structure but replaces
# the tool responses bigger than a certain size with a <the content has been removed> placeholder.


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
        window = history[-self.window_size :]
        orphans = []
        for item in window:
            if item.get("role") == "tool":
                tool_call_id = item.get("tool_call_id", "")
                found = False
                for other in window:
                    # quick and dirty way to check if the tool call id is in the other item
                    if item != other and tool_call_id in str(other):
                        found = True
                        break

                if not found:
                    orphans.append(item)

        if orphans:
            window = [item for item in window if item not in orphans]

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
