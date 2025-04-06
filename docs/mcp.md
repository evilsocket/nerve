# Model Context Protocol

Nerve supports [MCP](https://modelcontextprotocol.io/introduction) both as a client and as a server:

* [Client](#mcp-client)
* [Server](#mcp-server)
    - [Serving an Agent](#serving-an-agent)
    - [Serving an Agent and its Tools](#serving-an-agent-and-its-tools)
    - [Tools Only](#tools-only)

## MCP Client

An agent can use any server from [the multitude of publicly available MPC servers](https://github.com/punkpeye/awesome-mcp-servers) with:

```yaml
agent: You are a helpful assistant.
task: Write something to your knowledge graph, then read it back, save it to output.txt and set your task as complete.

using:
  - task

mcp:
  memory:
    command: npx
    args: ["-y", "@modelcontextprotocol/server-memory"]

  filesystem:
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "."]
```

## MCP Server

Nerve can also act as a MCP server via the `nerve serve <agent>` command. In this mode you can turn an agent into a tool for other agents to use.

### Serving an Agent

For instance this command line will serve the [code-audit](https://github.com/evilsocket/code-audit) agent as an MCP server:

```bash
nerve serve code-audit --mcp
```

This means that agents can use other agents as tools, for instance:

```yaml
agent: You are a helpful assistant.
task: Perform a code audit of {{ path }}.

using:
  - task

mcp:
  code_audit:
    command: nerve
    args: ["serve", "code-audit", "--mcp"]
```

You can find a full [example here](https://github.com/evilsocket/nerve/tree/main/examples/mcp-recipe).

### Serving an Agent and its Tools

Additionally you can also serve the agent tools:

```bash
nerve serve code-audit --mcp -t
```

This will export both the agent itself and its tools via MCP.

### Tools Only

You can use this mechanism to serve simple tools via MCP as well. For instance if you create a `tools.yml` like this:

```yaml
tools:
  - name: get_weather
    description: Get the current weather in a given place.
    arguments:
        - name: place
          description: The place to get the weather of.
          example: Rome
    tool: curl wttr.in/{{ place }}
```

You can serve it via MCP with:

```bash
nerve serve tools.yml --mcp
```