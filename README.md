<p align="center">
    <img src="assets/logo.svg" alt="nerve" width="300" align='center'/>
</p>

<p align="center">
  Create LLM agents in a simple YAML based syntax.
</p>

<p align="center">
  <a href="https://github.com/dreadnode/nerve/releases/latest"><img alt="Release" src="https://img.shields.io/github/release/dreadnode/nerve.svg?style=flat-square"></a>
  <a href="https://crates.io/crates/nerve-ai"><img alt="Crate" src="https://img.shields.io/crates/v/nerve-ai.svg"></a>
  <a href="https://hub.docker.com/r/dreadnode/nerve"><img alt="Docker Hub" src="https://img.shields.io/docker/v/dreadnode/nerve?logo=docker"></a>
  <a href="https://rust-reportcard.xuri.me/report/github.com/dreadnode/nerve"><img alt="Rust Report" src="https://rust-reportcard.xuri.me/badge/github.com/dreadnode/nerve"></a>
  <a href="#"><img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/dreadnode/nerve/test.yml"></a>
  <a href="https://github.com/dreadnode/nerve/blob/master/LICENSE.md"><img alt="Software License" src="https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square"></a>
</p>

<p align="center">
    <strong>
        <a href="https://github.com/dreadnode/nerve/blob/main/docs/index.md" target="_blank">
            Documentation
        </a>
    </strong>
</p>

- 🧑‍💻 **Agents Made Simple:** Agents are defined using YAML based files called [tasklets](https://github.com/dreadnode/nerve/blob/main/docs/tasklets.md). _The sky is the limit!_ You can define an agent for any task you desire — check out the [existing examples](https://github.com/dreadnode/nerve/tree/main/examples) for inspiration.
- 🧠 **Automated Problem Solving:** Nerve provides [a standard library of actions](https://github.com/dreadnode/nerve/blob/main/docs/namespaces.md) the agent uses autonomously to inform and enhance its performance. These include identifying specific goals required to complete the task, devising and revising a plan to achieve those goals, and creating and recalling memories comprised of pertinent information gleaned during previous actions.
- 🛠️ **Simple and Universal Tool Calling:** Nerve will automatically detect if the selected model natively supports function calling. If not, it will provide a compatibility layer that empowers the LLM to perform function calling anyway.
- 🤖 **Works with any LLM:** Nerve is an [LLM-agnostic tool](https://github.com/dreadnode/nerve/blob/main/docs/index.md#llm-support).
- 🤝 **Multi-Agent Workflows:** Nerve allows you to define a [multi-agent workflow](https://github.com/dreadnode/nerve/blob/main/docs/workflows.md), where each agent is responsible for a different part of the task.
- 💯 **Zero Code:** The project's main goal and core difference with other tools is to allow the user to instrument smart agents by writing simple YAML files.

## Usage

Please refer to the [documentation](https://github.com/dreadnode/nerve/blob/main/docs/index.md) for installation and usage instructions.

## License

Nerve is released under the GPL 3 license. To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.

[![Star History Chart](https://api.star-history.com/svg?repos=dreadnode/nerve&type=Date)](https://star-history.com/#dreadnode/nerve&Date)
