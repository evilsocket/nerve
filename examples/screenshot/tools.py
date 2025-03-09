import base64
import os
import shutil
import typing as t

from nerve.runtime import state


def take_screenshot() -> dict[str, str]:
    """
    Take a screenshot of the current screen.
    """

    screenshot_path = "/tmp/screenshot.png"
    if shutil.which("screencapture"):
        # macOS
        os.system(f"screencapture {screenshot_path}")
    elif shutil.which("gnome-screenshot"):
        os.system(f"gnome-screenshot -f {screenshot_path}")
    elif shutil.which("flameshot"):
        os.system(f"flameshot full -p {screenshot_path}")
    elif shutil.which("scrot"):
        os.system(f"scrot {screenshot_path}")
    else:
        raise Exception("No screenshot tool found")

    with open(screenshot_path, "rb") as image_file:
        base64_image = base64.b64encode(image_file.read()).decode("utf-8")

    return {
        "type": "image_url",
        "image_url": {"url": f"data:image/png;base64,{base64_image}"},
    }


def describe_screenshot(description: t.Annotated[str, "The description of the screenshot."]) -> None:
    """
    Describe the screenshot.
    """
    print(description)

    state.set_task_complete()
