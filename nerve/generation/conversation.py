import typing as t

from loguru import logger

from nerve.generation import WindowStrategy


class FullHistoryStrategy(WindowStrategy):
    """
    This strategy returns the full history of the conversation.
    """

    async def get_window(self, history: list[dict[str, t.Any]]) -> list[dict[str, t.Any]]:
        return history

    def __str__(self) -> str:
        return "<full history>"


class SlidingWindowStrategy(WindowStrategy):
    """
    This strategy returns a sliding window of the last N messages.
    """

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


class StrippedWindowStrategy(WindowStrategy):
    """
    This strategy returns a sliding window of the last N messages, but instead of removing
    earlier messages it will replace the content of the messages with a <stripped> placeholder.
    """

    def __init__(self, window_size: int = 10) -> None:
        self.window_size = window_size

    async def get_window(self, history: list[dict[str, t.Any]]) -> list[dict[str, t.Any]]:
        history_size = len(history)
        if history_size <= self.window_size:
            return history

        window = []
        # calculate which messages should be stripped (all except the last window_size messages)
        messages_to_strip = max(0, history_size - self.window_size)

        for current_index, msg in enumerate(history):
            # create a copy of the message to avoid modifying the original
            msg = msg.copy()

            # strip content for messages outside the window (earlier messages)
            if current_index < messages_to_strip:
                if msg.get("role") == "tool":
                    # For tool messages, preserve tool_call_id but strip content
                    msg["content"] = "<stripped tool response>"
                elif "content" in msg:
                    msg["content"] = "<stripped content>"

            window.append(msg)

        return window

    def __str__(self) -> str:
        return f"<stripping window of size {self.window_size}>"


def strategy_from_string(strategy: str) -> WindowStrategy:
    if strategy == "full":
        return FullHistoryStrategy()
    elif strategy.startswith("strip-") and strategy.split("-")[1].isdigit():
        return StrippedWindowStrategy(int(strategy.split("-")[1]))
    elif strategy.isdigit():
        return SlidingWindowStrategy(int(strategy))
    else:
        raise ValueError(f"Invalid conversation strategy: {strategy}")
