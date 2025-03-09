# Nerve: The Simple Agent Development Kit

<p align="center">
  <a href="https://github.com/evilsocket/nerve/releases/latest"><img alt="Release" src="https://img.shields.io/github/release/evilsocket/nerve.svg?style=flat-square"></a>
  <a href="https://pypi.org/project/nerve-adk"><img alt="Package" src="https://img.shields.io/pypi/v/nerve-adk.svg"></a>
  <a href="https://hub.docker.com/r/evilsocket/nerve"><img alt="Docker Hub" src="https://img.shields.io/docker/v/evilsocket/nerve?logo=docker"></a>
  <a href="#"><img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/evilsocket/nerve/ci.yml"></a>
  <a href="https://github.com/evilsocket/nerve/blob/master/LICENSE.md"><img alt="Software License" src="https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square"></a>
</p>

<p align="center">
  🧠 + 🛠️ = 🤖
</p>

Nerve is an ADK ( _Agent Development Kit_ ) with a convenient command line tool designed to be a simple yet powerful platform for creating and executing LLM-based agents.

🖥️ Install:

```bash
pip install nerve-adk
```

💡 Create an agent with the guided procedure:

```bash
nerve create new-agent
```

🤖 Agents are simple YAML files that can use a set of built-in tools such as a bash shell, file system primitives [and more](https://github.com/evilsocket/nerve/blob/main/docs/namespaces.md):

```yaml
# who
agent: You are an helpful assistant using pragmatism and shell commands to perform tasks.
# what
task: Find which running process is using more RAM.
# how
using: [task, bash]
```

🚀 Execute the agent with:

```bash
nerve run new-agent
```

🛠️ The agent capabilities can be extended directly via YAML (the [android-agent](https://github.com/evilsocket/nerve/blob/main/examples/android-agent) for a perfect example of this):

```yaml
tools:
  - name: get_weather
    description: Get the current weather in a given place.
    arguments: 
      - name: place
        description: The place to get the weather of.
        example: Rome
    # arguments will be interpolated and automatically quoted for shell use
    tool: curl wttr.in/{{ place }}
```

🐍 Or in Python, by adding a `tools.py` file, for more complex features (check this [webcam agent example](https://github.com/evilsocket/nerve/blob/main/examples/webcam)):

```python
import typing as t

# This annotated function will be available as a tool to the agent.
def read_webcam_image(foo: t.Annotated[str, "Describe arguments to the model like this."]) -> dict[str, str]:
    """Reads an image from the webcam."""

    base64_image = '...'
    return {
        "type": "image_url",
        "image_url": {"url": f"data:image/jpeg;base64,{base64_image}"},
    }
```

👨‍💻 Alternatively, you can use Nerve as a Python package and leverage its abstractions to create an entirely custom agent loop (see [the ADK examples](https://github.com/evilsocket/nerve/blob/main/examples/adk/)).

## Usage

Please refer to the [documentation](https://github.com/evilsocket/nerve/blob/main/docs/index.md) and the [examples](https://github.com/evilsocket/nerve/tree/main/examples).

## License

Nerve is released under the GPL 3 license.

[![Star History Chart](https://api.star-history.com/svg?repos=evilsocket/nerve&type=Date)](https://star-history.com/#evilsocket/nerve&Date)