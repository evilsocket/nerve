#!/usr/bin/env python3
import base64
import cv2
import os


def fetch_webcam_image(webcam_url):
    cap = cv2.VideoCapture(webcam_url)
    try:
        if cap.isOpened():
            ret, frame = cap.read()
            if ret:
                return frame
            else:
                raise Exception("Failed to read frame from RTSP stream")
        else:
            raise Exception("Could not open RTSP stream")
    finally:
        cap.release()


webcam_url = os.getenv("NERVE_WEBCAM_URL")
if webcam_url is None:
    raise Exception("NERVE_WEBCAM_URL environment variable is not set")

image = fetch_webcam_image(webcam_url)

# save the image to a file in the same directory as the script
script_path = os.path.abspath(__file__)
script_dir = os.path.dirname(script_path)
screenshot_path = os.path.join(script_dir, "webcam.png")

cv2.imwrite(screenshot_path, image)

# return the image as a base64 encoded string
with open(screenshot_path, "rb") as f:
    image_data = f.read()

print(base64.b64encode(image_data).decode("utf-8"))
