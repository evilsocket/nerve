"""
Computer use primitives for mouse, keyboard, and screen.
"""

import base64
import io
import typing as t

import pyautogui as px
import pyperclip

# this is an extra feature, so we need to indicate it
OPTIONAL_FEATURE = "computer_use"
# for docs
EMOJI = "💻"

# max screenshot width
MAX_WIDTH = 1280
# typing delay in ms
TYPING_DELAY_MS = 12

_width = 0
_height = 0
_display_num = 0
_scale_factor = 1.0
_target_width = 0
_target_height = 0
_scaling_enabled = False


def _init_scaling() -> None:
    global _width, _height, _display_num, _scale_factor, _target_width, _target_height

    if _width is None:
        screen_size = px.size()
        _width = int(screen_size[0])
        _height = int(screen_size[1])
        _display_num = None  # Not used on MacOS

        if _scaling_enabled and _width > MAX_WIDTH:
            _scale_factor = MAX_WIDTH / _width
            _target_width = MAX_WIDTH
            _target_height = int(_height * _scale_factor)
        else:
            _scale_factor = 1.0
            _target_width = _width
            _target_height = _height


def _scale_coordinates(x: int, y: int) -> tuple[int, int]:
    if not _scaling_enabled:
        return x, y

    x_scaling_factor = _width / _target_width
    y_scaling_factor = _height / _target_height

    return round(x / x_scaling_factor), round(y / y_scaling_factor)


async def screenshot() -> dict[str, t.Any]:
    """Take a screenshot of the current screen."""

    _init_scaling()

    screenshot = px.screenshot()
    if _scaling_enabled and _scale_factor < 1.0:
        screenshot = screenshot.resize((_target_width, _target_height))

    img_buffer = io.BytesIO()

    screenshot.save(img_buffer, format="PNG", optimize=True)
    img_buffer.seek(0)
    base64_image = base64.b64encode(img_buffer.read()).decode()

    return {
        "type": "image_url",
        "image_url": {"url": f"data:image/png;base64,{base64_image}"},
    }


async def get_cursor_position() -> str:
    """Get the current mouse position."""

    x, y = px.position()
    x, y = _scale_coordinates(int(x), int(y))

    return f"({x}, {y})"


async def mouse_move(
    x: t.Annotated[int, "The x coordinate to move to"],
    y: t.Annotated[int, "The y coordinate to move to"],
) -> None:
    """Move the mouse to the given coordinates."""

    px.moveTo(x, y)


async def mouse_left_click() -> None:
    """Click the left mouse button at the current mouse position."""

    px.click(button="left")


async def mouse_left_click_drag(
    x: t.Annotated[int, "The x coordinate to move to"],
    y: t.Annotated[int, "The y coordinate to move to"],
) -> None:
    """Click and drag the left mouse button from the current mouse position to the given coordinates."""

    px.mouseDown(button="left")
    px.moveTo(x, y)
    px.mouseUp(button="left")


async def mouse_right_click() -> None:
    """Click the right mouse button at the current mouse position."""

    px.click(button="right")


async def mouse_middle_click() -> None:
    """Click the middle mouse button at the current mouse position."""

    px.click(button="middle")


async def mouse_double_click() -> None:
    """Double click the left mouse button at the current mouse position."""

    px.doubleClick()


async def mouse_scroll(
    x: t.Annotated[int, "The x coordinate to move to"],
    y: t.Annotated[int, "The y coordinate to move to"],
) -> None:
    """Scroll the mouse wheel in the given direction."""

    px.scroll(x, y)


async def keyboard_press_hotkeys(
    keys: t.Annotated[str, "The hotkey sequence to press (like 'ctrl+shift+cmd+space')"],
) -> None:
    """Press one or more hotkeys on the keyboard."""

    key_sequence = list(map(str.strip, keys.lower().split("+")))
    replace = {
        "control": "ctrl",
        "option": "alt",
        "super": "command",
        "cmd": "command",
        "return": "enter",
        "escape": "esc",
        "spacebar": "space",
    }
    key_sequence = [replace.get(key, key) for key in key_sequence]

    px.hotkey(*key_sequence)


async def keyboard_type(text: t.Annotated[str, "The text to type"]) -> None:
    """Type the given text on the keyboard."""

    # px.typewrite(text, interval=TYPING_DELAY_MS / 1000.0)

    # https://github.com/asweigart/pyautogui/issues/259
    pyperclip.copy(text)
    await keyboard_press_hotkeys("cmd+v")
    pyperclip.copy("")
