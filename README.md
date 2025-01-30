<p align="center">
    <img src="assets/logo.svg" alt="nerve" width="300" align='center'/>
</p>

<p align="center">
  Create LLM agents without writing code.
</p>

<p align="center">
  <a href="https://github.com/dreadnode/nerve/releases/latest"><img alt="Release" src="https://img.shields.io/github/release/dreadnode/nerve.svg?style=flat-square"></a>
  <a href="https://crates.io/crates/nerve-ai"><img alt="Crate" src="https://img.shields.io/crates/v/nerve-ai.svg"></a>
  <a href="https://hub.docker.com/r/dreadnode/nerve"><img alt="Docker Hub" src="https://img.shields.io/docker/v/dreadnode/nerve?logo=docker"></a>
  <a href="https://rust-reportcard.xuri.me/report/github.com/dreadnode/nerve"><img alt="Rust Report" src="https://rust-reportcard.xuri.me/badge/github.com/dreadnode/nerve"></a>
  <a href="#"><img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/dreadnode/nerve/test.yml"></a>
  <a href="https://github.com/dreadnode/nerve/blob/master/LICENSE.md"><img alt="Software License" src="https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square"></a>
</p>

- 🧑‍💻 **User-Defined Agents:** Agents are defined using a YAML based files called `tasklets`. _The sky is the limit!_ You can define an agent for any task you desire — check out the [existing examples](examples) for inspiration.
- 🧠 **Automated Problem Solving:** Nerve provides a standard library of actions the agent uses autonomously to inform and enhance its performance. These include identifying specific goals required to complete the task, devising and revising a plan to achieve those goals, and creating and recalling memories comprised of pertinent information gleaned during previous actions.
- 🛠️ **Simple and Universal Tool Calling:** Nerve will automatically detect if the selected model natively supports function calling. If not, it will provide a compatibility layer that empowers the LLM to perform function calling anyway.
- 🤖 **Works with any LLM:** Nerve is an LLM-agnostic tool.
- 🤝 **Multi-Agent Workflows:** Nerve allows you to define a multi-agent `workflow`, where each agent is responsible for a different part of the task.
- 💯 **Zero Code:** The project's main goal and core difference with other tools is to allow the user to instrument smart agents by writing simple YAML files.

## LLM Support

Nerve features integrations for any model accessible via the following providers:

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

¹ **o1-preview and o1 models do not support function calling directly** and do not support a system prompt. Nerve will try to detect this and fallback to user prompt. It is possible to force this behaviour by adding the `--user-only` flag to the command line.

² Refer to [this document](https://huggingface.co/blog/tgi-messages-api#using-inference-endpoints-with-openai-client-libraries) for how to configure a custom Huggingface endpoint.

## Installing with Cargo

The easiest and recommended way to install Nerve is via [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```sh
cargo install nerve-ai
```

## Installing from DockerHub

Alternatively a Docker image is available on [Docker Hub](https://hub.docker.com/r/dreadnode/nerve). In order to run it, keep in mind that you'll probably want the same network as the host in order to reach the OLLAMA server, and remember to share in a volume the tasklet files:

```sh
docker run -it --network=host -v ./examples:/root/.nerve/tasklets dreadnode/nerve -h
```

## Building from sources

To build from source:

```sh
cargo build --release
```

## Usage

In order to use Nerve you need to specify which model to use trough a generator string (see the `LLM Support` section) and a tasklet file. 

For instance the command below will run the `examples/code_auditor` tasklet using the `gpt-4o` model from OpenAI:

```sh
nerve -G "openai://gpt-4o" -T examples/code_auditor 
```

Some tasklets require additional arguments that can be passed with `-D name=value` via the command line. For instance the `code_auditor` tasklet requires a `TARGET_PATH` argument:

```sh
nerve -G "openai://gpt-4o" -T examples/code_auditor -D TARGET_PATH=/path/to/code
```

In case of a workflow, you can specify the workflow file with the `-W`/`--workflow` argument:

```sh
nerve -W examples/recipe_workflow 
```

### Example Tasklet

Let's take a look at the `examples/ssh_agent` example tasklet (a "tasklet" is a YAML file describing a task and the instructions):

```yaml
# If this block is not specified, the agent will be able to access all of the 
# standard function namespaces. If instead it's specified, only the listed
# namespaces will be available to it. Use it to limit what the agent can do.
using:
  # the agent can save and recall memories
  - memory
  # the agent can update its own goal
  - goal
  # the agent can set the task as completed or impossible autonomously
  - task
  # the agent can create an action plan for the task
  - planning
  #  give the agent a sense of time
  - time

# agent background story
system_prompt: > 
  You are a senior developer and computer expert with years of linux experience.
  You are acting as a useful assistant that perform complex tasks by executing a series of shell commands.

# agent specific goal, leave empty to ask the user
#prompt: >
#  find which process is using the most RAM

# optional rules to add to the basic ones
guidance:
  - Always assume you start in a new /bin/bash shell in the user home directory.
  - Prefer using full paths to files and directories.
  - Use the /tmp directory for any file write operations.
  - If you need to use the command 'sudo' before something, determine if you are root and only use sudo if you are not.

# optional global action timeout
timeout: 120s

# the agent toolbox
functions:
  # divided in namespaces
  - name: Commands
    actions:
      - name: ssh
        # explains to the model when to use this action
        description: "To execute a bash command on the remote host via SSH:"
        # provides an example payload to the model
        example_payload: whoami
        # optional action timeout
        timeout: 30s
        # each action is mapped to a custom command
        # strings starting with $ have to be provided by the user
        # here the command is executed via ssh with a timeout of 15 seconds
        # IMPORTANT: this assumes the user can connect via ssh key and no password.
        tool: ssh $SSH_USER_HOST_STRING
```

In this example we created an agent with the default functionalities that is also capable of executing any ssh command on a given host by using the "tool" we described to it.

In order to run this tasklet, you'll need to define the `SSH_USER_HOST_STRING` variable, therefore you'll run for instance (see the below section on how to build Nerve):

```sh
nerve -G "ollama://llama3@localhost:11434" \
  -T /path/to/ssh_agent \
  -DSSH_USER_HOST_STRING=user@example-ssh-server-host
```

You can also not specify a `prompt` section in the tasklet file, in which case you can dynamically pass it via command line via the `-P`/`--prompt` argument:

```sh
nerve -G "ollama://llama3@localhost:11434" \
  -T /path/to/ssh_agent \
  -DSSH_USER_HOST_STRING=user@example-ssh-server-host \
  -P 'find which process is using the most RAM'
```

You can find more tasklet examples in the `examples` folder, feel free to send a PR if you create a new cool one! :D

### Robopages

Nerve can use functions from a [robopages server](https://github.com/dreadnode/robopages-cli). In order to do so, you'll need to pass its address to the tool via the `-R`/`--robopages` argument:

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

## License

Nerve is released under the GPL 3 license. To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.

[![Star History Chart](https://api.star-history.com/svg?repos=dreadnode/nerve&type=Date)](https://star-history.com/#dreadnode/nerve&Date)
