## Model Context Protocol (MCP)

Nerve has **first-class support** for [Model Context Protocol (MCP)](https://modelcontextprotocol.io/introduction), enabling agents to:

- 🔌 **Act as MCP clients** — consuming tools, memory, and capabilities exposed by external MCP servers.
- 🧩 **Act as MCP servers** — exposing their own tools or full agent behavior to be consumed by other agents.

🚀 Nerve is the **first tool** to offer YAML-based definition of MCP servers with seamless support for both client and server modes. This enables advanced **agent orchestration**, team-based delegation, and interoperability with the broader MCP ecosystem.

## ✅ MCP Client

An agent can use external tools, memory systems, or filesystems by defining `mcp:` in its YAML:

```yaml
agent: You are a helpful assistant.
task: Write something to your knowledge graph, then read it back, save it to output.txt, and mark the task complete.

using:
  - task

mcp:
  # for stdio based mcp servers
  memory:
    command: npx
    args: ["-y", "@modelcontextprotocol/server-memory"]

  filesystem:
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "."]

  # SSE
  example_sse:
    url: http://localhost:9090/

  # Streamable HTTP
  example_streamable_http:
    url: stream://localhost:8080/example
```

You can connect to any of the [publicly available MCP servers](https://github.com/punkpeye/awesome-mcp-servers), or define your own custom tools.

## 🖧 MCP Server

You can expose a Nerve agent **as a tool or agent** for other systems (or other agents) to call.
This enables:
- Modular pipelines
- Reusable agent services
- Team-like delegation structures

### Serve an Agent
To expose an agent via MCP:
```bash
nerve serve code-audit --mcp
```
Now other agents can use this as a remote tool.

### Use a Served Agent as Tool
Here’s how to use a served agent (`code-audit`) as a tool:
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

This will spin up the `code-audit` agent as a background server and allow your current agent to call it like a regular tool.

### Serve Tools + Agent
To expose both the agent and its tools:
```bash
nerve serve code-audit --mcp -t
```
This lets the remote caller decide whether to use the agent loop or call individual tools.

### Serve Tools Only
You can expose just a `tools.yml` file:
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

Then serve it via:
```bash
nerve serve tools.yml --mcp
```

## 🔁 Combining Client and Server
Nerve can run **as both a client and a server** in the same project. This means:
- You can define an agent that uses MCP tools, while being itself served as a tool.
- You can build hierarchical agent architectures, where a main agent delegates to sub-agents exposed via MCP.

This opens the door to **modular agent systems** and **secure service isolation**, where agents can be reused across workflows or teams.

## 🌐 Use Cases
- Local memory or filesystem integration for stateful agents
- Reusable audit/code/analysis agents callable by other teams
- Building team-like behaviors where each agent specializes in one task
- Sandboxing tools behind a network interface

For full examples, see the [mcp-recipe](https://github.com/evilsocket/nerve/tree/main/examples/mcp-recipe).

## 🧭 Related Docs
- [concepts.md](concepts.md#mcp-model-context-protocol) — overview of how MCP fits into Nerve's architecture
- [index.md](index.md) — quick usage examples
- [workflows.md](workflows.md) — for linear pipelines (MCP enables more complex ones)

