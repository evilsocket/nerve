## Nerve: The Simple Agent Development Kit

Nerve is an **Agent Development Kit (ADK)** that makes it easy to create and execute intelligent agents powered by LLMs, using a clean YAML-based syntax and a powerful CLI.

> For an overview of concepts, see [concepts.md](concepts.md).


### 📦 Installation

Requires Python 3.10+ and pip:
```bash
pip install nerve-adk
```
To upgrade:
```bash
pip install --upgrade nerve-adk
```
To uninstall:
```bash
pip uninstall nerve-adk
```

#### Docker Image
A Docker image is available at [Docker Hub](https://hub.docker.com/r/evilsocket/nerve):
```bash
docker run -it --network=host -v ./examples:/root/.nerve/agents evilsocket/nerve -h
```

## 🚀 Usage Overview

### Installing an Agent
You can install agents from GitHub or ZIP archives:
```bash
nerve install evilsocket/changelog
nerve run changelog
nerve uninstall changelog
```
Override the task:
```bash
nerve run changelog --task "use a single sentence for the changelog"
```

> [!TIP]  
> Nerve primarily loads agents from `$HOME/.nerve/agents`, ensuring that any agent in this folder is accessible regardless of your current working directory. If the agent is not found there, Nerve will then search in the current working directory as a fallback.

### Creating an Agent
Run the guided setup:
```bash
nerve create new-agent
```
It prompts you for:
- Location
- System prompt
- Task
- Tools (from the [built-in namespaces](namespaces.md))

> [!TIP]  
> You can use a `@` prefix for the system prompt as a shortcut to load the prompt from a markdown file in your `$HOME/.nerve/prompts` directory. For example, specifying `@scientist` as the system prompt will automatically load the prompt from either `$HOME/.nerve/prompts/scientist.md` or `$HOME/.nerve/prompts/scientist/system.md`.

Example output (`new-agent/agent.yml`):
```yaml
agent: You are a helpful assistant.
task: Make an HTTP request to {{ url }}
using:
  - shell
  - task
```
Run it with:
```bash
nerve run new-agent --url cnn.com
```

### Prompting & Variables
Supports [Jinja2](https://jinja.palletsprojects.com/) templating. You can:
- Include files: `{% include 'filename.md' %}`
- Interpolate args: `{{ url }}`
- Use built-in vars: `{{ CURRENT_DATE }}`, `{{ LOCAL_IP }}`, etc.
- Run tools inline: `{{ get_logs_tool() }}`

### Built-in Variables

```yaml
task: The current date is {{ CURRENT_DATE }} and the local IP is {{ LOCAL_IP }}.
```

**Date and Time**

| Symbol | Description |
|--|-|
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
|--|-|
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
|--|-|
| `LOCAL_IP` | Machine's local IP address on the network. |
| `PUBLIC_IP` | Public IP address as seen from the internet (requires internet connection). |
| `HOSTNAME` | Computer's network hostname. |

**Random Values**

| Symbol | Description |
|--|-|
| `RANDOM_INT` | Random integer between 0 and 10000. |
| `RANDOM_HEX` | Random 64-bit hexadecimal value. |
| `RANDOM_FLOAT` | Random floating point number between 0 and 1. |
| `RANDOM_STRING` | Random 10-character alphanumeric string. |

**System Integration**

| Symbol | Description |
|--|-|
| `CLIPBOARD` | Current content of the system clipboard. |


### 🧠 Models
Default model is OpenAI `gpt-4o-mini`. Override via env or CLI:
```bash
export NERVE_GENERATOR=anthropic/claude-3-7-sonnet
nerve run agent
```
Or pass it directly:
```bash
nerve run -g "ollama/llama3.2?temperature=0.9" agent
```
Set generator in YAML too:
```yaml
generator: "anthropic/claude"
```

Nerve supports all [LiteLLM providers](https://docs.litellm.ai/docs/providers).

### 🗣 Interactive Mode
Run in interactive step-by-step mode:
```bash
nerve run agent -i
```
Available commands:
- `step`, `s`, or `Enter`: one step
- `continue`, `c`: run till done
- `view`, `v`: view current state
- `quit`, `q`, `exit`: exit

### 🎥 Record & Replay
Record sessions:
```bash
nerve run agent --trace trace.jsonl
```
Replay sessions:
```bash
nerve play trace.jsonl
nerve play trace.jsonl -f  # fast-forward
```

### 🛠 Adding Tools
See [concepts.md](concepts.md#tools) for details.

Add shell tools:

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

Python tools go in a `tools.py` file next to the agent YAML:

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
Controls how much history the model sees:
- `full` (default): entire history
- `-c 5`: last 5 messages
- `-c strip-5`: full history, but only last 5 messages have content

```bash
nerve run agent -c full
nerve run agent -c 5
nerve run agent -c strip-5
```

### 🔌 MCP Integration
Nerve supports MCP (Model Context Protocol).
- As a **client** to use remote tools or memory
- As a **server** to expose agents and tools

See full details in [mcp.md](mcp.md).

### 📊 Evaluation Mode
Test agent performance on predefined test sets:
```bash
nerve eval path/to/eval --output results.json
```
Supports YAML, Parquet, or directory-based formats.
See [evaluation.md](evaluation.md).

### 🔄 Workflows
Workflows let you chain agents sequentially. Each agent receives inputs and contributes to shared state.
```bash
nerve run examples/recipe-workflow --food pizza
```

See [workflows.md](workflows.md) for a full breakdown.

For more complex orchestrations, see [concepts.md](concepts.md#workflows).

### 🕵️‍♂️ Observability / Tracing

Nerve supports tracing via LiteLLM supported observability providers such as [langfuse](https://docs.litellm.ai/docs/observability/langfuse_integration), [OpenTelemetry](https://docs.litellm.ai/docs/observability/opentelemetry_integration) and more.

In order to enable tracing with one of these providers, set the relevant environment variables and then pass the provider name via the `--litellm-tracing` command line argument:

```bash
# install tracing dependencies
pip install langfuse

# set environment variables
export LANGFUSE_PUBLIC_KEY="..."
export LANGFUSE_SECRET_KEY="..."
export LANGFUSE_HOST="..."

# execute an agent with tracing
nerve run <agent-name> --litellm-tracing langfuse
```

### 🧭 More
- [concepts.md](concepts.md): Core architecture & mental model
- [evaluation.md](evaluation.md): Agent testing & benchmarking
- [mcp.md](mcp.md): Advanced agent integration with MCP
- `namespaces.md`: (Coming soon)

Run `nerve -h` to explore all commands and flags.