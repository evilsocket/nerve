# Evaluation Mode

Nerve's **evaluation mode** is a strategic feature designed to make benchmarking and validating agents easy, reproducible, and formalized.

> ⚡ Unlike most tools in the LLM ecosystem, Nerve offers a built-in framework to **test agents against structured cases**, log results, and compare performance across models. It introduces a standard formalism for agent evaluation that does not exist elsewhere.


## 🎯 Why Use It?
Evaluation mode is useful for:
- Verifying agent correctness during development
- Regression testing when updating prompts, tools, or models
- Comparing different model backends
- Collecting structured performance metrics


## 🧪 Running an Evaluation
You run evaluations using:
```bash
nerve eval path/to/evaluation --output results.json
```
Each case is passed to the agent, and results (e.g., completion, duration, output) are saved.


## 🗂 Case Formats
Nerve supports three evaluation case formats:

### 1. `cases.yml`
For small test suites. Example:
```yaml
- level1:
    program: "A# #A"
- level2:
    program: "A# #B B# #A"
```
Used like this in the agent:
```yaml
task: >
  Consider this program:

  {{ program }}

  Compute it step-by-step and submit the result.
```

Used in [eval-ab](https://github.com/evilsocket/eval-ab).

### 2. `cases.parquet`
For large, structured datasets. Example from [eval-mmlu](https://github.com/evilsocket/eval-mmlu):
```yaml
task: >
  ## Question

  {{ question }}

  Use the `select_choice` tool to pick the right answer:
  {% for choice in choices %}
  - [{{ loop.index0 }}] {{ choice }}
  {% endfor %}
```

Can use HuggingFace datasets (e.g., MMLU) directly.

### 3. Folder-Based `cases/`
Organize each case in its own folder:
```
cases/
  level0/
    input.txt
  level1/
    input.txt
```
Useful when tools/scripts dynamically load inputs.
See [eval-regex](https://github.com/evilsocket/eval-regex).


## 🧪 Output
Results are written to a `.json` file with details like:
- Case identifier
- Task outcome (success/failure)
- Runtime duration
- Agent/tool outputs


## 📎 Notes
- You can define multiple runs per case for robustness
- Compatible with any agent setup (tools, MCP, workflows, etc.)
- All variables from each case are injected via `{{ ... }}`


## 🧭 Related Docs
- [concepts.md](concepts.md#evaluation)
- [index.md](index.md): CLI usage
- [mcp.md](mcp.md): when using remote agents or tools in evaluation

