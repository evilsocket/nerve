# Namespaces

Nerve offers a rich set of predefined tools, organized in namespaces, that the agent can import [via the `using` directive](index.md#usage). This page contains the list of namespaces available in Nerve, with the descriptive prompt that will be provided to the model.

## anytool

Let the agent create its own tools in Python.

| Tool | Description |
|------|-------------|
| `create_tool` | <pre>Create a new tool or redefine an existing one by defining it as an annotated Python function.<br>    Use this tool to implement the missing functionalities you need to perform your task.</pre> |

## filesystem

Read-only access primitives to the local filesystem.

| Tool | Description |
|------|-------------|
| `list_folder_contents` | <pre>List the contents of a folder on disk.</pre> |
| `read_file` | <pre>Read the contents of a file from disk.</pre> |

## reasoning

Simulates the reasoning process at runtime.

| Tool | Description |
|------|-------------|
| `clear_thoughts` | <pre>If the reasoning process proved wrong, inconsistent or ineffective, clear your thoughts and start again.</pre> |
| `think` | <pre>Adhere strictly to this reasoning framework, ensuring thoroughness, precision, and logical rigor.<br><br>    ## Problem Decomposition<br><br>    Break the query into discrete, sequential steps.<br>    Explicitly state assumptions and context.<br><br>    ## Stepwise Analysis<br><br>    Address each step individually.<br>    Explain the rationale, principles, or rules applied (e.g., mathematical laws, linguistic conventions).<br>    Use examples, analogies, or intermediate calculations to illustrate reasoning.<br><br>    ## Validation & Error Checking<br><br>    Verify logical consistency at each step.<br>    Flag potential oversights, contradictions, or edge cases.<br>    Confirm numerical accuracy (e.g., recompute calculations).<br><br>    ## Synthesis & Conclusion<br><br>    Integrate validated steps into a coherent solution.<br>    Summarize key insights and ensure the conclusion directly addresses the original query.</pre> |

## shell

Let the agent execute shell commands.

| Tool | Description |
|------|-------------|
| `execute_shell_command` | <pre>Execute a shell command and return the output.</pre> |

## task

Let the agent autonomously set the task as complete or failed.

| Tool | Description |
|------|-------------|
| `task_complete_success` | <pre>When your objective has been reached use this tool to set the task as complete.</pre> |
| `task_failed` | <pre>Use this tool if you determine that the given goal or task is impossible given the information you have.</pre> |

## time

Provides tools for getting the current date and time and waiting for a given number of seconds.

| Tool | Description |
|------|-------------|
| `current_time_and_date` | <pre>Get the current date and time.</pre> |
| `wait` | <pre>Wait for a given number of seconds.</pre> |

