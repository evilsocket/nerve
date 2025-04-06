# Nerve: The Simple Agent Development Kit

Nerve is an ADK ( _Agent Development Kit_ ) with a convenient command line tool designed to be a simple yet powerful platform for creating and executing LLM-based agents using a simple YAML-based syntax.

* [Installation](#installation)
* [Usage](#usage)
  - [Prompting](#prompting)
  - [Models](#-models)
  - [Interactive Mode](#-interactive-mode)
  - [Record & Replay](#-record--replay)
  - [Adding Tools](#️-adding-tools)️
  - [Conversation Window](#-conversation-window)
* [Model Context Protocol](mcp.md)
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

### Install an Agent

Starting from Nerve 1.4.x, agents can be installed from a github repository or zip archive URL. For instance, to install the [Changelog](https://github.com/evilsocket/changelog) agent you can run:

```bash
nerve install evilsocket/changelog
```

This will download, extract and install the agent to the folder `$HOME/.nerve/agents` allowing you to execute it with:

```bash
nerve run changelog
```

You can override the default task of any agent:

```bash
nerve run changelog --task 'use a single sentence for the changelog'
```

You can uninstall agents with:

```bash
nerve uninstall changelog
```

### Create an Agent

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

### Prompting

Both the `agent` and task `fields` support the JINJA2 template syntax, meaning you can:

Include local files:

```yaml
agent: {% include 'system_prompt.md' %}"
```

Preemptively execute a tool and use its output as part of the prompt:

```yaml
task: "Here are the logs: {{ get_logs_tool() }}"
```

Interpolate custom variables that the user will pass via command line argument:

```yaml
# passed via `nerve run agent-name --url ...`
task: Make an HTTP request to {{ url }}
```

Reference a set of built-in variables:

```yaml
task: The current date is {{ CURRENT_DATE }} and the local IP is {{ LOCAL_IP }}.
```

**Date and Time**

| Symbol | Description |
|--------|-------------|
| `CURRENT_DATE` | Current date in YYYY-MM-DD format. |
| `CURRENT_TIME` | Current time in HH:MM:SS format. |
| `CURRENT_DATETIME` | Current date and time in YYYY-MM-DD HH:MM:SS format. |
| `CURRENT_YEAR` | Current year (e.g., "2025"). |
| `CURRENT_MONTH` | Current month as a zero-padded number (e.g., "04" for April). |
| `CURRENT_DAY` | Current day of the month as a zero-padded number (e.g., "03"). |
| `CURRENT_WEEKDAY` | Current day of the week (e.g., "Thursday"). |
| `TIMEZONE` | Current timezone name (e.g., "EST", "UTC"). |
| `CURRENT_TIMESTAMP` | Current Unix timestamp (seconds since epoch). |

**Platform Information**

| Symbol | Description |
|--------|-------------|
| `USERNAME` | Current user's login name. |
| `PLATFORM` | Operating system name (e.g., "Windows", "Darwin", "Linux"). |
| `OS_VERSION` | Detailed operating system version information. |
| `ARCHITECTURE` | System architecture (e.g., "x86_64", "arm64"). |
| `PYTHON_VERSION` | Version of Python currently running. |
| `HOME` | Path to the current user's home directory. |
| `PROCESS_ID` | ID of the current process. |
| `WORKING_DIR` | Current working directory path. |

**Network Information**

| Symbol | Description |
|--------|-------------|
| `LOCAL_IP` | Machine's local IP address on the network. |
| `PUBLIC_IP` | Public IP address as seen from the internet (requires internet connection). |
| `HOSTNAME` | Computer's network hostname. |

**Random Values**

| Symbol | Description |
|--------|-------------|
| `RANDOM_INT` | Random integer between 0 and 10000. |
| `RANDOM_HEX` | Random 64-bit hexadecimal value. |
| `RANDOM_FLOAT` | Random floating point number between 0 and 1. |
| `RANDOM_STRING` | Random 10-character alphanumeric string. |

**System Integration**

| Symbol | Description |
|--------|-------------|
| `CLIPBOARD` | Current content of the system clipboard. |

### 🧠 Models

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

Alternatively, a generator string can be set via YAML directly:

```yaml
generator: 'anthropic/claude-3-7-sonnet-20250219'

agent: You are a helpful assistant.

# ...
```

### 🗣 Interactive Mode

By default, Nerve operates in automatic mode, allowing the agent loop to run continuously until an exit condition is met. To gain more control over the process and interact with the agent in real time as it executes steps, use the `-i` flag to enable interactive mode.

```sh
nerve run any-agent -i
```

In this mode, you will have access to the following commands at each step:

* **help** (alias `h`): show the help menu.
* **quit** (alias `q` or `exit`): stop the execution and exit.
* **step** (alias `s` or just hit enter with no command): execute a single step.
* **continue** (alias `c`, `cont`, or `go`): continue the execution until completion.
* **view** (alias `v`): inspect the current state.

Anything else will be interpreted and used as a chat message for the current agent.

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

Since version 1.5.0 Nerve is integrated with [Model Context Protocol (MCP)](https://modelcontextprotocol.io/introduction), you can refer to the [MCP section](mcp.md) of the documentation to use MCP tools.

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

Receive the entire conversation, but strip the contents of every message before the last N (with N=5 in this example):

```bash
nerve run agent -c strip-5
```