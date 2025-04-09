# Evaluation Mode

Nerve provides an evaluation mode that allows you to test your agent's performance against a set of predefined test cases. This is useful for:

- Validating agent behavior during development
- Regression testing after making changes
- Benchmarking different models
- Collecting metrics on agent performance

An evaluation consists of an agent and a corresponding set of test cases. These cases can be defined in a `cases.yml` file, stored in a `cases.parquet` file, or organized as individual entries within separate folders.

Regardless of how you organize the evaluation cases, the agent will be executed for each one, with a specified number of runs per case. Task completion data and runtime statistics will be collected and saved to an output file.

```bash
nerve eval path/to/evaluation --output results.json
```

## YAML

You can place a `cases.yml` file in the agent folder with the different test cases. For instance, this is used in the [ab evaluation](https://github.com/evilsocket/eval-ab), where the evaluation cases look like:

```yaml
- level1:
    program: "A# #A"
- level2:
    program: "A# #B B# #A"
# ... and so on
```

These cases are interpolated in the agent prompt:

```yaml
task: >
  ## Problem

  Now, consider the following program:

  {{ program }}

  Fully compute it, step by step and then submit the final result.
```

## Parquet

For more complex test suite you can use a `cases.parquet` file. An example of this is [this MMLU evaluation](https://github.com/evilsocket/eval-mmlu) that is loading data from the [MMLU (dev) dataset](https://huggingface.co/datasets/cais/mmlu) and using it in the agent prompt:

```yaml
task: >
  ## Question

  {{ question }}

  Use the select_choice tool to select the correct answer from this list of possible answers:

  {% for choice in choices %}
  - [{{ loop.index0 }}] {{ choice }}
  {% endfor %}
```

## Folders

You can also divide your cases in a `cases` folder in order like in [the regex evaluation](https://github.com/evilsocket/eval-regex) where each input file is organized in `ccases/level0`, `cases/level1`, etc and [read at runtime](https://github.com/evilsocket/eval-regex/blob/main/tools.py#L11) by the tools.