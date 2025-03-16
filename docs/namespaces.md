# Namespaces

Nerve offers a rich set of predefined tools, organized in namespaces, that the agent can import [via the `using` directive](index.md#usage). This page contains the list of namespaces available in Nerve, with the descriptive prompt that will be provided to the model.

## 🔧 anytool

Let the agent create its own tools in Python.

<details>
<summary><b>Show Tools</b></summary>

### `create_tool`

<pre>Create a new tool or redefine an existing one by defining it as an annotated Python function.
    Use this tool to implement the missing functionalities you need to perform your task.</pre>

**Parameters**

* `code` <i>(<class 'str'>)</i>: The Python code to create the tool.

</details>

## 💻 computer

> [!IMPORTANT]
> This namespace is not available by default and requires the `computer_use` optional feature.
> To enable it, run `pip install nerve-adk[computer_use]`.

Computer use primitives for mouse, keyboard, and screen.

<details>
<summary><b>Show Tools</b></summary>

### `get_cursor_position`

<pre>Get the current mouse position.</pre>

### `keyboard_press_hotkeys`

<pre>Press one or more hotkeys on the keyboard.</pre>

**Parameters**

* `keys` <i>(<class 'str'>)</i>: The hotkey sequence to press (like 'ctrl+shift+cmd+space')

### `keyboard_type`

<pre>Type the given text on the keyboard.</pre>

**Parameters**

* `text` <i>(<class 'str'>)</i>: The text to type

### `mouse_double_click`

<pre>Double click the left mouse button at the current mouse position.</pre>

### `mouse_left_click`

<pre>Click the left mouse button at the current mouse position.</pre>

### `mouse_left_click_drag`

<pre>Click and drag the left mouse button from the current mouse position to the given coordinates.</pre>

**Parameters**

* `x` <i>(<class 'int'>)</i>: The x coordinate to move to
* `y` <i>(<class 'int'>)</i>: The y coordinate to move to

### `mouse_middle_click`

<pre>Click the middle mouse button at the current mouse position.</pre>

### `mouse_move`

<pre>Move the mouse to the given coordinates.</pre>

**Parameters**

* `x` <i>(<class 'int'>)</i>: The x coordinate to move to
* `y` <i>(<class 'int'>)</i>: The y coordinate to move to

### `mouse_right_click`

<pre>Click the right mouse button at the current mouse position.</pre>

### `mouse_scroll`

<pre>Scroll the mouse wheel in the given direction.</pre>

**Parameters**

* `x` <i>(<class 'int'>)</i>: The x coordinate to move to
* `y` <i>(<class 'int'>)</i>: The y coordinate to move to

### `screenshot`

<pre>Take a screenshot of the current screen.</pre>

</details>

## 📂 filesystem

Read-only access primitives to the local filesystem.

### Jail

The tools in this namespace can be restricted to a specific set of paths by using the `jail` directive in the agent configuration:

```yaml
using:
  - filesystem

jail:
    filesystem:
      - "/only/path/to/allow"
      - "{{ target_path }}" # variables can be used too
```

<details>
<summary><b>Show Tools</b></summary>

### `list_folder_contents`

<pre>List the contents of a folder on disk.</pre>

**Parameters**

* `path` <i>(<class 'str'>)</i>: The path to the folder to list

### `read_file`

<pre>Read the contents of a file from disk.</pre>

**Parameters**

* `path` <i>(<class 'str'>)</i>: The path to the file to read

</details>

## 📂 filesystem_w

Write primitives to the local filesystem.

### Jail

The tools in this namespace can be restricted to a specific set of paths by using the `jail` directive in the agent configuration:

```yaml
using:
  - filesystem_w

jail:
    filesystem_w:
      - "/only/path/to/allow"
      - "{{ target_path }}" # variables can be used too
```

<details>
<summary><b>Show Tools</b></summary>

### `create_file`

<pre>Create a file on disk, if the file already exists, it will be overwritten.</pre>

**Parameters**

* `path` <i>(<class 'str'>)</i>: The path to the file to create
* `content` <i>(str | None)</i>: The content to write to the file, if not provided, the file will be created empty

### `delete_file`

<pre>Delete a file from disk.</pre>

**Parameters**

* `path` <i>(<class 'str'>)</i>: The path to the file to delete

</details>

## 💬 inquire

Let the agent interactively ask questions to the user in a structured way.

<details>
<summary><b>Show Tools</b></summary>

### `ask_for_confirmation`

<pre>Ask a confirmation question to the user.</pre>

**Parameters**

* `question` <i>(<class 'str'>)</i>: The question to ask the user.
* `default` <i>(<class 'bool'>)</i>: The default answer to the question.

### `ask_for_multiple_choice`

<pre>Ask a multiple choice question to the user.</pre>

**Parameters**

* `question` <i>(<class 'str'>)</i>: The question to ask the user.
* `choices` <i>(list[str])</i>: The choices to offer the user.

### `ask_for_single_choice`

<pre>Ask a single choice question to the user.</pre>

**Parameters**

* `question` <i>(<class 'str'>)</i>: The question to ask the user.
* `choices` <i>(list[str])</i>: The choices to offer the user.

### `ask_question`

<pre>Ask a question to the user.</pre>

**Parameters**

* `question` <i>(<class 'str'>)</i>: The question to ask the user.

</details>

## 🧠 reasoning

Simulates the reasoning process at runtime.

<details>
<summary><b>Show Tools</b></summary>

### `clear_thoughts`

<pre>If the reasoning process proved wrong, inconsistent or ineffective, clear your thoughts and start again.</pre>

### `think`

<pre>Adhere strictly to this reasoning framework, ensuring thoroughness, precision, and logical rigor.

    ## Problem Decomposition

    Break the query into discrete, sequential steps.
    Explicitly state assumptions and context.

    ## Stepwise Analysis

    Address each step individually.
    Explain the rationale, principles, or rules applied (e.g., mathematical laws, linguistic conventions).
    Use examples, analogies, or intermediate calculations to illustrate reasoning.

    ## Validation & Error Checking

    Verify logical consistency at each step.
    Flag potential oversights, contradictions, or edge cases.
    Confirm numerical accuracy (e.g., recompute calculations).

    ## Synthesis & Conclusion

    Integrate validated steps into a coherent solution.
    Summarize key insights and ensure the conclusion directly addresses the original query.</pre>

**Parameters**

* `thought` <i>(<class 'str'>)</i>: A thought to think about

</details>

## 💻 shell

Let the agent execute shell commands.

<details>
<summary><b>Show Tools</b></summary>

### `shell`

<pre>Execute a shell command on the local computer and return the output. Non interactive shell with a timeout of 30 seconds.</pre>

**Parameters**

* `command` <i>(<class 'str'>)</i>: The shell command to execute

</details>

## ✅ task

Let the agent autonomously set the task as complete or failed.

<details>
<summary><b>Show Tools</b></summary>

### `task_complete_success`

<pre>When your objective has been reached use this tool to set the task as complete.</pre>

**Parameters**

* `reason` <i>(str | None)</i>: Optional reason why the task is complete or report of conclusive information.

### `task_failed`

<pre>Use this tool if you determine that the given goal or task is impossible given the information you have.</pre>

**Parameters**

* `reason` <i>(<class 'str'>)</i>: The reason why the task is impossible

</details>

## 🕒 time

Provides tools for getting the current date and time and waiting for a given number of seconds.

<details>
<summary><b>Show Tools</b></summary>

### `current_time_and_date`

<pre>Get the current date and time.</pre>

### `wait`

<pre>Wait for a given number of seconds.</pre>

**Parameters**

* `seconds` <i>(<class 'int'>)</i>: The number of seconds to wait

</details>

