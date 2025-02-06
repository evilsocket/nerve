An agent that monitors a webcam and informs the user if anything interesting is happening.

### Example Usage

This agent uses one tool to monitor the webcam and another one to create a UI to inform the user. 

In order to install their dependencies, run:

```sh
pip install -r path/to/webcam/requirements.txt
```

After that, you'll need to set the `NERVE_WEBCAM_URL` environment variable to the URL of the webcam you want to monitor, for instance:

```sh
export NERVE_WEBCAM_URL="rtsp://192.168.50.14:554/stream1"
```

For a list of supported formats, refer to the [OpenCV documentation](https://docs.opencv.org/4.x/dd/d43/tutorial_py_video_display.html).

Finally, you can run the agent with:

```sh
nerve -G "openai://gpt-4o" -T webcam --window 5
```

It is recommended to use [a conversation window size](https://github.com/dreadnode/nerve/blob/main/docs/tasklets.md#agent-loop-and-conversation-window) of 5 messages.