# Nerve

Nerve is a tool for building LLM agents using a simple YAML-based syntax. Agents are composed of [tasklets](tasklets.md) - modular steps that execute sequentially using tools available from [the standard library](namespaces.md) or a user-defined library. Furthermore tasklets can be orchestrated into [workflows](workflows.md) to create multi-agent environments to solve more complex tasks.

* [Installation](#installation)
* [Usage](#usage)
* [LLM Support](#llm-support)
* [Using with Robopages](#using-with-robopages)
* [Tasklets](tasklets.md)
* [Namespaces](namespaces.md)
* [Workflows](workflows.md)

## Installation

### Installing with Cargo

The easiest and recommended way to install Nerve is with [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```sh
cargo install nerve-ai
```

### Installing from DockerHub

Alternatively, a Docker image is available on [Docker Hub](https://hub.docker.com/r/dreadnode/nerve). In order to run it, you'll probably want the same network as the host in order to reach the OLLAMA server, and remember to share in a volume the tasklet files:

<!-- This snippet needs a bit more clarity. Perhaps splitting the sentences would help? The piece about "remember to share in a volume the tasklet files" is a little confusing. Does the user need to share the network? I think I need some help understanding this piece! -->

```sh
docker run -it --network=host -v ./examples:/root/.nerve/tasklets dreadnode/nerve -h
```

### Building from sources

To build from source:

<!-- Is "source" a standard term that our users are comfortable with? Do we need to explain what we mean by "sources" a bit more here? -->

```sh
cargo build --release
```

## Usage

In order to use Nerve, you must specify which model to use through a generator string and a tasklet file.

To specify the model, refer to the [LLM Support](#llm-support) and [tasklets](tasklets.md) documentation.

For instance the command below will run the `examples/code_auditor` tasklet using the `gpt-4o` model from OpenAI:

```sh
nerve -G "openai://gpt-4o" -T examples/code_auditor 
```

Some tasklets require additional arguments that can be passed with `-D name=value` using the command line. For instance, the `code_auditor` tasklet requires a `TARGET_PATH` argument:

```sh
nerve -G "openai://gpt-4o" -T examples/code_auditor -D TARGET_PATH=/path/to/code
```

In case of a workflow, you can specify the workflow file with the `-W`/`--workflow` argument:

<!-- "In case of a workflow", does this mean "when creating a workflow", "when using a workflow"? needs a tab more clarity here! -->

```sh
nerve -W examples/recipe_workflow 
```

### LLM Support

Nerve features integrations for any model accessible through the following providers:

| Name | API Key Environment Variable | Generator Syntax |
|----------|----------------------------|------------------|
| **Ollama** | - | `ollama://llama3@localhost:11434` |
| **Groq** | `GROQ_API_KEY` | `groq://llama3-70b-8192` |
| **OpenAI**¹ | `OPENAI_API_KEY` | `openai://gpt-4` |
| **Fireworks** | `LLM_FIREWORKS_KEY` | `fireworks://llama-v3-70b-instruct` |
| **Huggingface**² | `HF_API_TOKEN` | `hf://tgi@your-custom-endpoint.aws.endpoints.huggingface.cloud` |
| **Anthropic** | `ANTHROPIC_API_KEY` | `anthropic://claude` |
| **Nvidia NIM** | `NIM_API_KEY` | `nim://nvidia/nemotron-4-340b-instruct` |
| **DeepSeek** | `DEEPSEEK_API_KEY` | `deepseek://deepseek-chat` |
| **xAI** | `XAI_API_KEY` | `xai://grok-beta` |
| **Mistral.ai** | `MISTRAL_API_KEY` | `mistral://mistral-large-latest` |
| **Novita** | `NOVITA_API_KEY` | `novita://meta-llama/llama-3.1-70b-instruct` |
| **Google Gemini**³ | `GEMINI_API_KEY` | `gemini://gemini-2.0-flash` |

¹ **o1-preview and o1 models do not support function calling directly** and do not support a system prompt. Nerve will attempt to detect this and will fallback to the user prompt. It's possible to force this behaviour by adding the `--user-only` flag to the command line.

² To configure a custom Huggingface endpoint, refer to this document at: https://huggingface.co/blog/tgi-messages-api#using-inference-endpoints-with-openai-client-libraries.

³ Google Gemini OpenAI endpoint breaks with multiple tools. While this bug won't be fixed, Nerve will detect this and use its own xml based tooling prompt to work around this issue. To learn more about this issue, read the materials at: https://discuss.ai.google.dev/t/invalid-argument-error-using-openai-compatible/51788.

### Using with Robopages

Nerve can use functions from a [robopages server](https://github.com/dreadnode/robopages-cli). In order to do so, you'll need to pass its address to the tool using the `-R`/`--robopages` argument:

```sh
nerve -G "openai://gpt-4o" \
  -T /path/to/tasklet \
  -R "localhost:8000"
```

To import only a subset of tools:

```sh
nerve -G "openai://gpt-4o" \
  -T /path/to/tasklet \
  -R "localhost:8000/cybersecurity/reverse-engineering"
```
