<div align="center">

# `nerve`

<i>The Simple Agent Development Kit</i>

[![Release](https://img.shields.io/github/release/evilsocket/nerve.svg?style=flat-square)](https://github.com/evilsocket/nerve/releases/latest)
[![Package](https://img.shields.io/pypi/v/nerve-adk.svg)](https://pypi.org/project/nerve-adk)
[![Docker](https://img.shields.io/docker/v/evilsocket/nerve?logo=docker)](https://hub.docker.com/r/evilsocket/nerve)
[![CI](https://img.shields.io/github/actions/workflow/status/evilsocket/nerve/ci.yml)](https://github.com/evilsocket/nerve/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square)](https://github.com/evilsocket/nerve/blob/master/LICENSE.md)

</div>

Nerve is an ADK ( _Agent Development Kit_ ) designed to be a simple yet powerful platform for creating and executing LLM-based agents.

## Quick Start

```bash
# 🖥️ install the project with:
pip install nerve-adk

# 💡 create an agent with a guided procedure:
nerve create new-agent

# 🚀 go!
nerve run new-agent
```

Agents are simple YAML files that can use a set of built-in tools such as a bash shell, file system primitives [and others](https://github.com/evilsocket/nerve/blob/main/docs/namespaces.md):

```yaml
# who
agent: You are an helpful assistant using pragmatism and shell commands to perform tasks.
# what
task: Find which running process is using more RAM.
# how
using: [shell]
```

See the [documentation](https://github.com/evilsocket/nerve/blob/main/docs/index.md) and the [examples](https://github.com/evilsocket/nerve/tree/main/examples) for more.

## Contributing

We welcome contributions! Check out our [contributing guidelines](https://github.com/evilsocket/nerve/blob/main/CONTRIBUTING.md) to get started and join our [Discord community](https://discord.gg/btZpkp45gQ) for help and discussion.

## License

Nerve is released under the GPL 3 license.

[![Star History Chart](https://api.star-history.com/svg?repos=evilsocket/nerve&type=Date)](https://star-history.com/#evilsocket/nerve&Date)
