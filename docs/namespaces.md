# Namespaces

Nerve offers a rich set of predefined tools, organized in namespaces, that the agent can import [via the `using` directive](index.md#usage). This page contains the list of namespaces available in Nerve, with the descriptive prompt that will be provided to the model.

## anytool

Let the agent create its own tools in Python.

<details>
<summary><b>Show Tools</b></summary>
| Tool | Description |
|------|-------------|
| `create_tool` | <pre>Create a new tool or redefine an existing one by defining it as an annotated Python function.<br>    Use this tool to implement the missing functionalities you need to perform your task.</pre> |
</details>

## computer

> [!IMPORTANT]
> This namespace is not available by default and requires the `computer_use` optional feature.
> To enable it, run `pip install nerve-adk[computer_use]`.

Computer use primitives for mouse, keyboard, and screen.

<details>
<summary><b>Show Tools</b></summary>
| Tool | Description |
|------|-------------|
| `get_cursor_position` | <pre>Get the current mouse position.</pre> |
| `keyboard_press_hotkeys` | <pre>Press one or more hotkeys on the keyboard.</pre> |
| `keyboard_type` | <pre>Type the given text on the keyboard.</pre> |
| `mouse_double_click` | <pre>Double click the left mouse button at the current mouse position.</pre> |
| `mouse_left_click` | <pre>Click the left mouse button at the current mouse position.</pre> |
| `mouse_left_click_drag` | <pre>Click and drag the left mouse button from the current mouse position to the given coordinates.</pre> |
| `mouse_middle_click` | <pre>Click the middle mouse button at the current mouse position.</pre> |
| `mouse_move` | <pre>Move the mouse to the given coordinates.</pre> |
| `mouse_right_click` | <pre>Click the right mouse button at the current mouse position.</pre> |
| `mouse_scroll` | <pre>Scroll the mouse wheel in the given direction.</pre> |
| `screenshot` | <pre>Take a screenshot of the current screen.</pre> |
</details>

## filesystem

Read-only access primitives to the local filesystem.

<details>
<summary><b>Show Tools</b></summary>
| Tool | Description |
|------|-------------|
| `list_folder_contents` | <pre>List the contents of a folder on disk.</pre> |
| `read_file` | <pre>Read the contents of a file from disk.</pre> |
</details>

## inquire

Let the agent interactively ask questions to the user in a structured way.

<details>
<summary><b>Show Tools</b></summary>
| Tool | Description |
|------|-------------|
| `ask_for_confirmation` | <pre>Ask a confirmation question to the user.</pre> |
| `ask_for_multiple_choice` | <pre>Ask a multiple choice question to the user.</pre> |
| `ask_for_single_choice` | <pre>Ask a single choice question to the user.</pre> |
| `ask_question` | <pre>Ask a question to the user.</pre> |
</details>

## reasoning

Simulates the reasoning process at runtime.

<details>
<summary><b>Show Tools</b></summary>
| Tool | Description |
|------|-------------|
| `clear_thoughts` | <pre>If the reasoning process proved wrong, inconsistent or ineffective, clear your thoughts and start again.</pre> |
| `think` | <pre>Adhere strictly to this reasoning framework, ensuring thoroughness, precision, and logical rigor.<br><br>    ## Problem Decomposition<br><br>    Break the query into discrete, sequential steps.<br>    Explicitly state assumptions and context.<br><br>    ## Stepwise Analysis<br><br>    Address each step individually.<br>    Explain the rationale, principles, or rules applied (e.g., mathematical laws, linguistic conventions).<br>    Use examples, analogies, or intermediate calculations to illustrate reasoning.<br><br>    ## Validation & Error Checking<br><br>    Verify logical consistency at each step.<br>    Flag potential oversights, contradictions, or edge cases.<br>    Confirm numerical accuracy (e.g., recompute calculations).<br><br>    ## Synthesis & Conclusion<br><br>    Integrate validated steps into a coherent solution.<br>    Summarize key insights and ensure the conclusion directly addresses the original query.</pre> |
</details>

## shell

Let the agent execute shell commands.

<details>
<summary><b>Show Tools</b></summary>
| Tool | Description |
|------|-------------|
| `shell` | <pre>Execute a shell command on the local computer and return the output. Non interactive shell with a timeout of 30 seconds.</pre> |
</details>

## task

Let the agent autonomously set the task as complete or failed.

<details>
<summary><b>Show Tools</b></summary>
| Tool | Description |
|------|-------------|
| `task_complete_success` | <pre>When your objective has been reached use this tool to set the task as complete.</pre> |
| `task_failed` | <pre>Use this tool if you determine that the given goal or task is impossible given the information you have.</pre> |
</details>

## time

Provides tools for getting the current date and time and waiting for a given number of seconds.

<details>
<summary><b>Show Tools</b></summary>
| Tool | Description |
|------|-------------|
| `current_time_and_date` | <pre>Get the current date and time.</pre> |
| `wait` | <pre>Wait for a given number of seconds.</pre> |
</details>

