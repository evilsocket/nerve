import base64
import os
import textwrap
import time
import typing as t

import cv2


def read_webcam_image() -> dict[str, str]:
    """Reads an image from the webcam."""

    webcam_url = os.getenv("NERVE_WEBCAM_URL")
    if webcam_url is None:
        raise Exception("NERVE_WEBCAM_URL environment variable is not set")

    cap = cv2.VideoCapture(webcam_url)
    try:
        if cap.isOpened():
            ret, frame = cap.read()
            if ret:
                # save the image to a file in the same directory as the script
                script_path = os.path.abspath(__file__)
                script_dir = os.path.dirname(script_path)
                screenshot_path = os.path.join(script_dir, "webcam.jpg")
                cv2.imwrite(screenshot_path, frame)

                with open(screenshot_path, "rb") as image_file:
                    base64_image = base64.b64encode(image_file.read()).decode("utf-8")

                return {
                    "type": "image_url",
                    "image_url": {"url": f"data:image/jpeg;base64,{base64_image}"},
                }

            else:
                raise Exception("Failed to read frame from RTSP stream")
        else:
            raise Exception("Could not open RTSP stream")
    finally:
        cap.release()


def inform_user(message: t.Annotated[str, "The message to inform the user about."]) -> None:
    """Use this tool to inform the user about interesting activity."""

    # read the current webcam image from disk
    script_path = os.path.abspath(__file__)
    script_dir = os.path.dirname(script_path)
    screenshot_path = os.path.join(script_dir, "webcam.jpg")
    image = cv2.imread(screenshot_path)

    # add text with wrapping
    height, width, channel = image.shape
    font = cv2.FONT_HERSHEY_SIMPLEX
    wrapped_text = textwrap.wrap(message, width=35)
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
    output_path = os.path.join(script_dir, f"{time.time()}.jpg")
    cv2.imwrite(output_path, image)

    # show the image on a UI
    cv2.imshow("image", image)
    cv2.waitKey(0)
    cv2.destroyAllWindows()
