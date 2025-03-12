# Nerve: The Simple Agent Development Kit

Nerve is an ADK ( _Agent Development Kit_ ) with a convenient command line tool designed to be a simple yet powerful platform for creating and executing LLM-based agents using a simple YAML-based syntax.

* [Installation](#installation)
* [Usage](#usage)
  - [Generators](#-generators)
  - [Record & Replay](#-record--replay)
  - [Adding Tools](#️-adding-tools)️
  - [Conversation Window](#-conversation-window)
* [Namespaces](namespaces.md)
* [Workflows](workflows.md)

## 💻 Installation

Nerve requires Python 3.10 or later and PIP. You can install it with:

```bash
pip install nerve-adk
```

To upgrade to the latest version, run:

```bash
pip install --upgrade nerve-adk
```

For a list of options:

```bash
nerve -h
```

To uninstall, run:

```bash
pip uninstall nerve-adk
```

### 🐋 Installing from DockerHub

Alternatively, a Docker image is available on [Docker Hub](https://hub.docker.com/r/evilsocket/nerve). When running the container, if you're using a local inference server such as OLLAMA, you'll likely want it to share the same network as the host. To allow Nerve to reach network endpoints like OLLAMA, use the command below. This ensures the container runs without network isolation and can access the same resources on the network as your host computer.

Additionally, remember to share the tasklet files by mounting a volume when running the container.

```sh
docker run -it --network=host -v ./examples:/root/.nerve/agents evilsocket/nerve -h
```

## 🚀 Usage

Nerve agents are simple YAML files that can use a set of built-in tools such as a bash shell, file system primitives [and more](https://github.com/evilsocket/nerve/blob/main/docs/namespaces.md). 

You can start creating an agent with a guided procedure by executing:

```bash
nerve create new-agent
```

During this procedure, you'll be prompted for the following:

- where to save this agent (either as a folder or single yml file).
- the agent **system prompt**, which is what determines the agent role (the `You are a useful assistant ...` stuff).
- the agent **task** - what does this agent has to do? (`what is 4+3?`)
- which **tools** from [the built-in namespaces](https://github.com/evilsocket/nerve/blob/main/docs/namespaces.md) the agent can use to perform its task.

> [!TIP]  
> You can use a `@` prefix for the system prompt as a shortcut to load the prompt from a markdown file in your `$HOME/.nerve/prompts` directory. For example, specifying `@scientist` as the system prompt will automatically load the prompt from either `$HOME/.nerve/prompts/scientist.md` or `$HOME/.nerve/prompts/scientist/system.md`.

After completing the procedure, your `new-agent/agent.yml` file will look something like this:

```yaml
agent: You are a helpful assistant.

# jinja2 templating syntax supported
task: Make an HTTP request to {{ url }}

using:
- shell # can execute shell commands
- task # can autonomously set the task as complete or failed
```

To run this agent (the `--url` is required because we referenced it in the agent `task`):

```bash
# equivalent to "nerve run new-agent/agent.yml"
nerve run new-agent --url 'cnn.com'
```

For a list of all the subcommands and their options, feel free to explore `nerve -h`.

> [!TIP]  
> Nerve primarily loads agents from `$HOME/.nerve/agents`, ensuring that any agent in this folder is accessible regardless of your current working directory. If the agent is not found there, Nerve will then search in the current working directory as a fallback.

### 🧠 Generators

The default model is OpenAI `gpt-4o-mini`, in order to use a different model you can either set the `NERVE_GENERATOR` environment variable, or pass it as a generator string via the `-g/--generator` command line argument.

> [!NOTE]  
> Nerve supports any of the LiteLLM supported providers, check [the litellm documentation](https://docs.litellm.ai/docs/providers) for a list of all the providers and their syntax.

```sh
export ANTHROPIC_API_KEY=sk-ant-api...
export NERVE_GENERATOR=anthropic/claude-3-7-sonnet-20250219

nerve run new-agent --url 'cnn.com'
```

is equivalent to:

```sh
nerve run -g "anthropic/claude-3-7-sonnet-20250219" new-agent --url 'cnn.com'
```

To pass additional inference parameters:

```sh
nerve run -g "ollama/llama3.2?temperature=0.9&api_base=http://server-host:11434" new-agent --url 'cnn.com'
```

### 🎥 Record & Replay

Nerve sessions can be recorded to a JSONL file by specifying the `--trace` argument:

```sh
nerve run new-agent --trace agent-trace.jsonl
```

The session can then be replayed at any time:

```sh
nerve play agent-trace.jsonl # this plays the session at the original speed
```

Add `-f` to replay in fast forward mode:

```sh
nerve play agent-trace.jsonl -f # much faster
```

### 🛠️ Adding Tools

When a tool can be represented as a shell command, you can conveniently extend the agent capabilites in the YAML:

```yaml
agent: You are a helpful assistant.
task: What's the weather in {{ city }}?

tools:
  # this adds the get_weather tool
  - name: get_weather
    # it's important to let the agent know what the tool purpose is.
    description: Get the current weather in a given place.
    arguments:
        # named arguments with descriptions and examples
        - name: place
          description: The place to get the weather of.
          example: Rome
    # arguments will be interpolated by name and automatically quoted for shell use
    tool: curl wttr.in/{{ place }}
```

If the tool requires more advanced capabilities, you can implement it in Python, by adding a `tools.py` file in the same folder of the agent with annotated functions:

```python
# new-agent/tools.py

import typing as t

# This annotated function will be available as a tool to the agent.
def read_webcam_image(foo: t.Annotated[str, "Describe arguments to the model like this."]) -> dict[str, str]:
    """Reads an image from the webcam."""

    # a tool can return a simple scalar value, or a dictionary for models with vision.
    base64_image = '...'
    return {
        "type": "image_url",
        "image_url": {"url": f"data:image/jpeg;base64,{base64_image}"},
    }
```

### 💬 Conversation Window

An agent will continue running in a loop execute tools at each step until one of the following conditions is met:

- The task status has been set as complete (either by the agent itself if using the `task` namespace, or by one of the tools if its `complete_task` field is `true`).
- The task status has been set as failed (by the agent itself if using the `task` namespace)
- The task times out.

During this loop the chat history the model has access to is defined by the conversation window, default to `full`, determined by the `--conversation/c` argument.

Full conversation window (the default behaviour, the model receives the entire conversation history):

```bash
nerve run agent -c full
```

Only receive the last N messages (with N=5 in this example):

```bash
nerve run agent -c 5
```