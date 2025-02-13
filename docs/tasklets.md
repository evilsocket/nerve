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

Tasklets are the building blocks of agents. They are defined in YAML files and specify the system and user prompts, the tasks the agent can perform, and optional guidelines for completing those tasks. Tasklet examples are available ([here](https://github.com/nerve-ai/nerve/tree/main/examples)).

A tasklet dynamically defines the chat history used to generate the agent's response in the agent loop:

```python
# pseudo code
while not task.done:
    history = tasklet.prepare_chat_history()
    agent.step(history)
```

The agent will continue running and make decisions at each step until one of the following conditions is met:

- The task is complete and has been set as such.
- The task is impossible and has been set as such.
- The task times out.
- One of the tools with the `complete_task` flag set to `true` is executed.

At each step the agent will receive a conversation history, composed of the initial prompts, the model tool calls, and their outputs. The size of this window is determined by the `--window` argument:

- If the window is set to a number `N`, the most recent `N` messages will be shown. **This is the default behaviour with N=15.**
- If the window is set to `full`, the entire conversation history will be shown to the agent.
- If the window is set to `summary`, the most recent messages will be shown, while the previous ones will be stripped down.

## Prompts

The two essential blocks of a tasklet are the system and user prompts. The system prompt is a description of the agent's role, while the user prompt defines the specific task the agent will perform:

```yaml
system_prompt: You are a helpful assistant.

prompt: How much is 2 + 2?
```

If you don't provide a user prompt, the agent will ask for the prompt at runtime.

## Guidance

You can provide a set of rules to help the agent to perform the task. These rules are called guidance:

```yaml
system_prompt: You are a helpful assistant.

prompt: How much is 2 + 2?

guidance:
    - Reason step by step.
    - Make sure your answer is correct.
    - Always be polite and professional.
```

## Task Timeout

You can set a timeout for the task. If the agent does not complete the task within the time period, the agent will be interrupted and the task will be marked as failed:

```yaml
# ... snippet ...

# timeout in seconds
timeout: 10 

# ... snippet ...
```

## AutoRAG

You can use a `rag` directive to automatically generate an index from a set of documents. This will allow the agent to use the RAG index to answer questions and use it to perform the specified task.

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

Nerve offers a rich set of predefined tools, organized in [namespaces](namespaces.md), that the agent can import with the `using` directive:

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

Although it may be tempting to include all items in the directive, it's important to remember that using more tools increases the number of tokens in the prompt. Smaller models, in particular, can become confused if too much information is provided at once. Therefore, it's best to use only the essential tools.

For more information about the default namespaces view the [namespaces documentation](namespaces.md).

### Custom Tools

Additional tools can be defined in the tasklet's `functions` section. Each tool is a set of actions that the agent can use, consisting of a `name`, a `description`, and a `tool` field that specifies the command to be executed:

```yaml
# ... snippet ...

functions:
  - name: News
    decription: You will use this action to read the recent news.
    actions:
      - name: read_news
        description: "To read the recent news:"
        # the output of this command will be returned to the agent
        tool: curl -s getnews.tech/ai,nocolor

# ... snippet ...
```

If the agent must provide arguments to the tool, you can define an `example_payload` to instruct the agent on how to use the tool:

```yaml
# ... snippet ...
functions:
  - name: Environment
    actions:
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

If the tool requires named arguments, you can define them in the `args` field:

```yaml
# ... snippet ...
functions:
  - name: Conversation
    description: You will use these actions to create conversational entries.
    actions:
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

- `max_shown_output`: The maximum number of characters to be shown in the output of the tool.
- `store_to`: Save the output of the tool in a named variable used to pass data between different tasks. View [example workflows](https://github.com/search?q=repo%3Adreadnode%2Fnerve+store_to+language%3AYAML&type=code).
- `timeout`: Timeout for the specific tool.
- `mime_type`: If set to `image/<any valid format>`, like `image/png`, the output of the tool will be considered as a base64 encoded image for vision models. View examples for [screenshots](https://github.com/dreadnode/nerve/tree/main/examples/screenshot) and [webcams](https://github.com/dreadnode/nerve/tree/main/examples/webcam).
- `complete_task`: If set to `true`, the task will be marked as complete after the tool is executed.
- `judge`: Use another tasklet as a judge to validate the output of the tool. View an example of a [code auditor with judge](https://github.com/dreadnode/nerve/tree/main/examples/code_auditor_with_judge).
- `alias`: Create a tool that's an alias to one of the predefined ones. View examples of a [docker agent](https://github.com/dreadnode/nerve/tree/main/examples/docker-agent).
- `ignore_stderr`: If set to `true`, the `stderr` of the tool will be ignored.

## Evaluators

An evaluator is a command line that receives the current state of the agent through standard input and performs some evaluation. At the end of evaluation, the evaluator can:

1. Exit with a 42 status code if the task is completed successfully.
2. Exit with any other status code if the task is not completed successfully.
3. Print your output to stdout. The evaluation script will automatically add your console output to the chat history as feedback to the agent.

Review the [eval_test](https://github.com/dreadnode/nerve/tree/main/examples/eval_test) and [ab_problem](https://github.com/dreadnode/nerve/tree/main/examples/ab_problem) tasklets for complete examples.

