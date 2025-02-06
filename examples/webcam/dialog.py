#!/usr/bin/env python3
import cv2
import os
import sys
import textwrap
import time

text = sys.argv[1]

# read the current webcam image from disk
script_path = os.path.abspath(__file__)
script_dir = os.path.dirname(script_path)
screenshot_path = os.path.join(script_dir, "webcam.png")
image = cv2.imread(screenshot_path)

# add text with wrapping
height, width, channel = image.shape
font = cv2.FONT_HERSHEY_SIMPLEX
wrapped_text = textwrap.wrap(text, width=35)
x, y = 10, 40
font_size = 1
font_thickness = 2

for i, line in enumerate(wrapped_text):
    textsize = cv2.getTextSize(line, font, font_size, font_thickness)[0]

    gap = textsize[1] + 10
    y = int((image.shape[0] + textsize[1]) / 2) + i * gap + 350
    x = int((image.shape[1] - textsize[0]) / 2)

    cv2.putText(
        image,
        line,
        (x, y),
        font,
        font_size,
        (0, 0, 250),
        font_thickness,
        lineType=cv2.LINE_AA,
    )

# save the image to another file to preserve it
output_path = os.path.join(script_dir, f"{time.time()}.png")
cv2.imwrite(output_path, image)

# show the image on a UI
cv2.imshow("image", image)
cv2.waitKey(0)
cv2.destroyAllWindows()
