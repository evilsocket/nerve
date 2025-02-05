#!/usr/bin/env python3
import os
import base64
import shutil

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

with open(screenshot_path, "rb") as f:
    image_data = f.read()

print(base64.b64encode(image_data).decode("utf-8"))
