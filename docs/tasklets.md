# Tasklets

* [Agent Loop and Conversation Window](#agent-loop-and-conversation-window)
* [Prompts](#prompts)
* [Guidance](#guidance)
* [Task Timeout](#task-timeout)
* [AutoRAG](#autorag)
* [Tools](#tools)
    * [Predefined Tools](#predefined-tools)
    * [Custom Tools](#custom-tools)
        * [Additional Fields](#additional-fields)
* [Evaluators](#evaluators)

---

## Agent Loop and Conversation Window

Tasklets are the building blocks of agents. They are defined in YAML files (several [examples are available here](https://github.com/nerve-ai/nerve/tree/main/examples)) and provide the system and user prompts, what the agent can do and optional guidelines on how to perform the task. A tasklet defines dynamically the chat history that will be used to generate the agent's response in the agent loop:

```python
# pseudo code
while not task.done:
    history = tasklet.prepare_chat_history()
    agent.step(history)
```

The agent will keep running and take decisions at each step until one of the following conditions is met:

- the task is complete and has been set as such
- the task is impossible and has been set as such
- the task times out
- one of the tools with the `complete_task` flag set to `true` is executed

At each step the agent will receive a conversation history, composed of the initial prompts, the model tool calls and their outputs. The size of this window is determined by the `--window` argument:

- if the window is set to a number `N`, the most recent `N` messages will be shown. **This is the default behaviour with N=15.**
- if the window is set to `full`, the entire conversation history will be shown to the agent.
- if the window is set to `summary`, the most recent messages will be shown, while the previous ones will be stripped down.

## Prompts

The two essential blocks of a tasklet are the system and user prompts. The system prompt is a description of the agent's role, while the user prompt defines the specific task to be performed:

```yaml
system_prompt: You are a helpful assistant.

prompt: How much is 2 + 2?
```

If the prompt is not provided it will be asked to the user at runtime.

## Guidance

It is possible to provide a set of rules to help the agent to perform the task. These are called guidance:

```yaml
system_prompt: You are a helpful assistant.

prompt: How much is 2 + 2?

guidance:
    - Reason step by step.
    - Make sure your answer is correct.
    - Be always polite and professional.
```

## Task Timeout

It is possible to set a timeout for the task. If the agent does not complete the task within the timeout, it will be interrupted and the task will be marked as failed:

```yaml
# ... snippet ...

# timeout in seconds
timeout: 10 

# ... snippet ...
```

## AutoRAG

It is possible to use a `rag` directive to automatically generate an index from a set of documents. This will allow the agent to use the RAG index to answer questions and use it to perform the task at hand.

```yaml
# ... snippet ...

rag:
  # documents to import
  source_path: ./docs
  # rag persistent data path
  data_path: ./data
  # uncomment to enable chunking
  # chunk_size: 1023

# ... snippet ...
```

See the [auto_rag example](https://github.com/dreadnode/nerve/tree/main/examples/auto_rag) for a complete example.

## Tools

One of the main characteristics of an agent is the ability to use tools.

### Predefined Tools

Nerve offers a rich set of predefined tools, organized in [namespaces](namespaces.md), that the agent can import via the using directive:

```yaml
using:
    # gives access to the filesystem
    - filesystem
    # allows the agent to define goals and track progress
    - goal
    # gives access to http requests
    - http
    # allows the agent to store and retrieve memories
    - memory
    # allows the agent to create and implement plans
    - planning
    # gives the agent access to an index of documents for RAG
    - rag
    # allows shell commands to be executed
    - shell
    # allows the agent to set the task as complete or impossible autonomously
    # NOTE: this is the only namespace that is not imported by default
    - task
    # tells the agent the current time
    - time

# ... snippet ...
```

While it might be tempting to use all of them, it is important to remember that the more tools you use, the more tokens will be used in the prompt. Smaller models especially tend to get confused if too much information is provided at once, so it is recommended to use only the necessary ones.

For more information about the default namespaces see [the namespaces documentation](namespaces.md).

### Custom Tools

Additional tools can be defined in the tasklet's `tool_box` section, and each is a group of tools that can be used by the agent, defining a `name`, `description` and a `tool` field with the command to be executed:

```yaml
# ... snippet ...

tool_box:
  - name: News
    decription: You will use this action to read the recent news.
    tools:
      - name: read_news
        description: "To read the recent news:"
        # the output of this command will be returned to the agent
        tool: curl -s getnews.tech/ai,nocolor

# ... snippet ...
```

If the agent must provide arguments to the tool, it is possible to define an example_payload to instruct the agent on how to use the tool:

```yaml
# ... snippet ...
tool_box:
  - name: Environment
    tools:
      - name: report_finding
        description: When you are ready to report findings, use this tool for each finding.
        example_payload: >
          {
            "title": "SQL Injection",
            "description": "Short description of the vulnerability",
            "file": "path/to/vulnerable_file.py",
            "function": "function_name",
            "line": 123
          }
        tool: curl -s -XPOST -H Content-Type:application/json http://dropship/output -d
# ... snippet ...
```

If the tool requires named arguments it is possible to define them in the `args` field:

```yaml
# ... snippet ...
tool_box:
  - name: Conversation
    description: You will use these actions to create conversational entries.
    tools:
      - name: talk
        description: "To have one of the characters say a specific sentence:"
        example_payload: hi, how are you doing today?
        tool: ./talk.py
        # in case the command requires arguments, declare them with an example value
        args:
          character_name: NameOfTheSpeakingCharacter
# ... snippet ...
```

#### Additional Fields

In addition to the ones already mentioned, tools can optionally define the following fields:

- `max_shown_output`: the maximum number of characters to be shown in the output of the tool.
- `store_to`: save the output of the tool in a named variable used to pass data between different tasks (see the [example workflows](https://github.com/search?q=repo%3Adreadnode%2Fnerve+store_to+language%3AYAML&type=code)).
- `timeout`: timeout for the specific tool.
- `mime_type`: if set to `image/<any valid format>`, like `image/png`, the output of the tool will be considered as a base64 encoded image for vision models (see [examples/screenshot](https://github.com/dreadnode/nerve/tree/main/examples/screenshot) and [examples/webcam](https://github.com/dreadnode/nerve/tree/main/examples/webcam)).
- `complete_task`: if set to `true`, the task will be marked as complete after the tool is executed.
- `judge`: uses another tasklet as a judge to validate the output of the tool (see [examples/code_auditor_with_judge](https://github.com/dreadnode/nerve/tree/main/examples/code_auditor_with_judge))
- `alias`: use to create a tool that's an alias to one of the predefined ones (see [examples/docker-agent](https://github.com/dreadnode/nerve/tree/main/examples/docker-agent))
- `ignore_stderr`: if set to `true`, the stderr of the tool will be ignored.

## Evaluators

An evaluator is a command line that receives the current state of the agent via standard input and performs some evaluation, at the end of which it can:

1. Exit with a 42 status code if the task is completed successfully.
2. Exit with any other status code if the task is not completed successfully.
3. Return via stdout anything, that'll go to the chat history itself as feedback for the agent.

Check the [eval_test](https://github.com/dreadnode/nerve/tree/main/examples/eval_test) and [ab_problem](https://github.com/dreadnode/nerve/tree/main/examples/ab_problem) tasklets for complete examples.

