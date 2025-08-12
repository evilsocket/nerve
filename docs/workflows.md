# Workflows

Workflows in Nerve allow you to **chain multiple agents in a simple, linear sequence**. Each agent in a workflow executes in order, passing shared state (variables) to the next. This is ideal for tasks that can be split into clear, sequential steps.

> 📌 For more advanced orchestration (parallel execution, sub-agents, branching logic), it's recommended to use [MCP](mcp.md) and expose agents as tools to a primary orchestrator.

## 📋 Example: Recipe Workflow

In this example, a workflow chains four agents to:
1. Generate a list of ingredients
2. Describe preparation steps
3. Estimate prep time
4. Rewrite the result in an engaging way

You can run it with:
```bash
nerve run examples/recipe-workflow --food pizza
```

### YAML Definition
```yaml
name: "Write a Recipe"
description: "A workflow for writing a recipe."

actors:
  create_list_of_ingredients:
    generator: anthropic://claude

  describe_preparation_steps:
    generator: openai://gpt-4o

  estimate_time:
    generator: openai://gpt-4o-mini

  rewrite_nicely:
    generator: openai://gpt-4o
```
Each key in `actors` defines an agent in the pipeline, executed in sequence.

## 🧠 Agent Files
Each agent is a standalone YAML with a task and tools. These are simplified below:

### `create_list_of_ingredients.yml`
```yaml
agent: You are a talented chef.
task: prepare a list of ingredients for {{ food }}
using:
  - reasoning

tools:
  - name: create_list_of_ingredients
    description: Provide ingredients, one per line.
    complete_task: true
    arguments:
      - name: ingredients
        description: Ingredients list.
        example: >
          - 1 cup flour
          - 2 eggs
```

### `describe_preparation_steps.yml`
```yaml
agent: You are a creative chef.
task: describe the preparation steps for {{ food }} using {{ ingredients }}
using:
  - reasoning

tools:
  - name: describe_preparation_steps
    description: Preparation steps.
    complete_task: true
    arguments:
      - name: steps
        example: >
          - Mix ingredients
          - Bake for 20 minutes
```

### `estimate_time.yml`
```yaml
agent: Estimate total prep time using the given ingredients and steps.
task: Estimate time to prepare {{ food }} using:
  {{ ingredients }}
  {{ steps }}
using:
  - reasoning

tools:
  - name: estimate_time
    description: Estimated preparation time
    complete_task: true
    arguments:
      - name: preparation_time
        example: 25 minutes
```

### `rewrite_nicely.yml`
```yaml
agent: You are a food blogger.
task: Rewrite the recipe for {{ food }} in an engaging way.

Includes:
  - {{ preparation_time }}
  - {{ ingredients }}
  - {{ steps }}

tools:
  - name: rewrite
    description: Final blog-ready recipe
    complete_task: true
    arguments:
      - name: recipe
        example: >
          # **Epic Lazy Day Pancakes**
          [ ... ]
```

## 🔄 State and Variables

### Variable Passing
Variables are shared across all agents in a workflow. Each agent can:
- Read variables set by previous agents
- Set new variables for subsequent agents
- Override existing variables

### Default Values
Each agent can define defaults that apply when variables aren't set:
```yaml
defaults:
  temperature: 0.7
  max_tokens: 1000
```

### Variable Interpolation
All text fields support Jinja2 templating:
```yaml
task: Process {{ food }} with {{ ingredients }}
```

## 📎 Notes
- Agents receive inputs from previous agents via templating variables (e.g., `{{ ingredients }}`)
- Each tool must call `complete_task: true` to advance the workflow
- All agents can define their own generators, tools, and prompts independently

## 🧭 Related Docs
- [concepts.md](concepts.md#workflows)
- [mcp.md](mcp.md): for building advanced orchestrations
- [index.md](index.md): CLI usage overview
