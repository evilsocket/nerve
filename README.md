<div align="center">

# `nerve`

<i>The Simple Agent Development Kit</i>

[![Documentation](https://img.shields.io/badge/documentation-blue)](https://github.com/evilsocket/nerve/blob/main/docs/index.md)
[![Release](https://img.shields.io/github/release/evilsocket/nerve.svg?style=flat-square)](https://github.com/evilsocket/nerve/releases/latest)
[![Package](https://img.shields.io/pypi/v/nerve-adk.svg)](https://pypi.org/project/nerve-adk)
[![Docker](https://img.shields.io/docker/v/evilsocket/nerve?logo=docker)](https://hub.docker.com/r/evilsocket/nerve)
[![CI](https://img.shields.io/github/actions/workflow/status/evilsocket/nerve/ci.yml)](https://github.com/evilsocket/nerve/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square)](https://github.com/evilsocket/nerve/blob/master/LICENSE.md)

  <small>Join the project community on our server!</small>
  <br/><br/>
  <a href="https://discord.gg/https://discord.gg/btZpkp45gQ" target="_blank" title="Join our community!">
    <img src="https://dcbadge.limes.pink/api/server/https://discord.gg/btZpkp45gQ"/>
  </a>

</div>

Nerve is a simple yet powerful Agent Development Kit (ADK) to build, run, evaluate, and orchestrate LLM-based agents using just YAML and a CLI. It’s designed for technical users who want programmable, auditable, and reproducible automation using large language models.

## Key Features

**📝 Declarative Agents**

Define agents using a clean YAML format: system prompt, task, tools, and variables — all in one file.

**🔧 Built-in Tools & Extensibility**

Use shell commands, Python functions, or remote tools to power your agents. Tools are fully typed and annotated.

**🌐 Native MCP Support (Client & Server)**  

Nerve is the first framework to let you define **MCP servers in YAML** — and act as both **client and server**, enabling agent teams and [deep orchestration](https://github.com/evilsocket/nerve/blob/main/docs/mcp.md).

**📊 Evaluation Mode**  

[Benchmark your agents](https://github.com/evilsocket/nerve/blob/main/docs/evaluation.md) with YAML, Parquet, or folder-based test cases. Run reproducible tests, log structured outputs, and track regression or progress. 

**🔁 Workflows**  

Compose agents into simple, linear pipelines to create multi-step automations with shared context.

**🧪 LLM-Agnostic**  

Built on [LiteLLM](https://docs.litellm.ai/), Nerve supports OpenAI, Anthropic, Ollama, [and dozens more](https://docs.litellm.ai/docs/providers) — switch models in one line.

## Quick Start

```bash
# 🖥️ install the project with:
pip install nerve-adk

# ⬇️ download and install an agent from a github repo with:
nerve install evilsocket/changelog

# 💡 or create an agent with a guided procedure:
nerve create new-agent

# 🚀 go!
nerve run new-agent
```

Read the [documentation](https://github.com/evilsocket/nerve/blob/main/docs/index.md) and the [examples](https://github.com/evilsocket/nerve/tree/main/examples) for more.

## Contributing

We welcome contributions! Check out our [contributing guidelines](https://github.com/evilsocket/nerve/blob/main/CONTRIBUTING.md) to get started and join our [Discord community](https://discord.gg/btZpkp45gQ) for help and discussion.

## License

Nerve is released under the GPL 3 license.

[![Star History Chart](https://api.star-history.com/svg?repos=evilsocket/nerve&type=Date)](https://star-history.com/#evilsocket/nerve&Date)
