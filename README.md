Nerve is a tool that allows creating stateful agents with any LLM of your choice - without writing a single line of code. 
The tool provides to the model a framework of functionalities for planning, saving or recalling memories, etc (you can think about it as a "standard library" of functions for the LLM to use) by dynamically adapting the prompt and making it stateful over multiple inferences. The model will be able to access and use these functionalities in order to accomplish the task you provided.

<p align="center">
  <img alt="Nerve" src="https://raw.githubusercontent.com/evilsocket/nerve/main/image.jpg"/>
</p>

While Nerve was inspired by other projects such as [Dreadnode's Rigging framework](https://github.com/dreadnode/rigging), its main goal and core difference with other tools is to allow the user to instrument smart agents without writing code (unless required for custom functionalities).

**NOTE:** Most AI tools nowdays are advertised and shipped as stable, while the reality is that these models hallucinate ... **a lot**. Nerve is an experimental tool. Its API is subject to changes at any time before a stable release is reached. While it is still a valuable learning and experimenting resource, using it in production environments and/or in unsupervised contextes is discouraged. To have an idea of the project readiness, you can `grep -r TODO src` :)

## Example

Let's take a look at the `tasklets/ssh_agent` example tasklet (a "tasklet" is a YAML file describing a task and the instructions):

```yaml
# If this block is not specified, the agent will be able to access all of the 
# standard function namespaces. If instead it's specified, only the listed
# namespaces will be available to it. Use it to limit what the agent can do.
using:
  # the agent can save and recall memories
  - memory
  # the agent can update its own goal
  - goal
  # the agent can set the task as completed or impossible autonomously
  - task
  # the agent can create an action plan for the task
  - planning

# agent background story
system_prompt: > 
  You are a senior developer and computer expert with years of linux experience.
  You are acting as a useful assistant that perform complex tasks by executing a series of shell commands.

# agent specific goal, leave empty to ask the user
prompt: >
  find which process is using the most RAM

# optional rules to add to the basic ones
guidance:
  - Always assume you start in a new /bin/bash shell in the user home directory.
  - Prefer using full paths to files and directories.
  - Use the /tmp directory for any file write operations.
  - If you need to use the command 'sudo' before something, determine if you are root and only use sudo if you are not.

# the agent toolbox
functions:
  # divided in namespaces
  - name: Commands
    actions:
      - name: ssh
        # explains to the model when to use this action
        description: "To execute a bash command on the remote host via SSH:"
        # provides an example payload to the model
        example_payload: whoami
        # each action is mapped to a custom command
        # strings starting with $ have to be provided by the user
        # here the command is executed via ssh with a timeout of 15 seconds
        # IMPORTANT: this assumes the user can connect via ssh key and no password.
        tool: gtimeout -v $SSH_TIMEOUT||15 /usr/bin/ssh $SSH_USER_HOST_STRING
```

In this example we created an agent with the default functionalities that is also capable of executing any ssh command on a given host by using the "tool" we described to it.

In order to run this tasklet, you'll need an [OLLAMA server](https://ollama.ai/) with the model of your choice, and to define the `SSH_USER_HOST_STRING` variable. Therefore you'll run (see the below section on how to build Nerve):

```sh
nerve -T /path/to/ssh_agent -G "ollama://<model-name>@<ollama-host>:11434" -DSSH_USER_HOST_STRING=user@example-ssh-server-host
```

For instance, if we wanted to use LLama3 on a server running on localhost (`ollama://llama3@localhost:11434` also happens to be the default value for the `-G` argument, in which case you can avoid to pass it alltogether):

```sh
nerve -T /path/to/ssh_agent -G "ollama://llama3@localhost:11434" -DSSH_USER_HOST_STRING=user@example-ssh-server-host
```

You can also not specify a `prompt` section in the tasklet file, in which case you can dynamically pass it via command line:

```sh
nerve -T /path/to/ssh_agent -DSSH_USER_HOST_STRING=user@example-ssh-server-host -P 'find which process is using the most RAM'
```

You can find more examples in the `tasklets` folder, feel free to send a PR if you create a new cool one! :D

### How does it work?

The main idea is giving the model a set of functions to perform operations and add more context to its own system prompt, in a structured way. Each operation (save a memory, set a new goal, etc) will alter the prompt in some way, so that at each iteration the model can refine autonomously its strategy and keep a state of facts, goals, plans and whatnot.

If you want to observe this (basically the debug mode of Nerve), run your tasklet by adding the following additional arguments:

```sh
nerve -T whatever-tasklet --save-to state.txt --full-dump
```

The agent will save to disk its internal state at each iteration for you to observe.

## Building from sources

```sh
cargo build --release
```

Run a tasklet with a given OLLAMA server:

```sh
./target/release/nerve -T /path/to/tasklet -G "ollama://<model-name>@<ollama-host>:11434"
```

## Building with Docker

```sh
docker build . -t nerve
```

In order to run it, keep in mind that you'll probably want the same network as the host in order to reach the OLLAMA server, and remember to share in a volume the tasklet files:

```sh
docker run -it --network=host -v /path/to/your/tasklet:/app/tasklet nerve -h
```

## License

Nerve is released under the GPL 3 license. To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.
