# Workflows

Workflows allow you to orchestrate multiple agents in a sequence of steps. Each agent can be executed by a different model, if desired, and the "state" data is passed between them. Each agent will have its own prompt and tools, acting as an independent functional unit.

For instance, the [recipe workflow example](https://github.com/evilsocket/nerve/tree/main/examples/recipe-workflow) shows how to orchestrate four different agents to:

- Create a list of ingredients.
- Describe the preparation steps.
- Estimate the preparation time.
- Rewrite the recipe in a more engaging and interesting way.

You can execute it with:

```bash
nerve run examples/recipe-workflow --food pizza
```

Its YAML definition is:

```yaml
name: "Write a Recipe"
description: "A workflow for writing a recipe."

flow:
  create_list_of_ingredients:
    generator: anthropic://claude

  describe_preparation_steps:
    generator: openai://gpt-4o

  estimate_time:
    generator: openai://gpt-4o-mini

  rewrite_nicely:
    generator: openai://gpt-4o
```

Each element of the `flow` array is an agent that will be executed in the order specified.

**create_list_of_ingredients.yml**

This first agent will create a list of ingredients that will be saved in the `ingredients` variable. Once the tool is executed, the task will be marked as complete and the next tasklet will execute.

```yaml
agent: You are a talented chef.

task: prepare a list of ingredients for {{ food }}

using:
  - reasoning

tools:
  - name: create_list_of_ingredients
    description: "To provide the ingredients one per line as an organized list:"
    complete_task: true # sets the task as complete when this tool is executed
    arguments:
      - name: ingredients
        description: The ingredients to create a list of.
        example: >
          - 1 cup of flour
          - 1 cup of sugar
          - 1 cup of eggs
          - 1 cup of milk
          - 1 cup of butter
```

**describe_preparation_steps.yml**

This tasklet will use the `ingredients` variable, by interpolating it in the prompt with the `$` symbol, to describe the preparation steps.

```yaml
agent: You are a talented and creative chef.

task: describe the preparation steps for {{ food }}. The ingredients at your disposal are {{ ingredients }}.

using:
  - reasoning

tools:
  - name: describe_preparation_steps
    description: "To provide the preparation steps one per line as an organized list:"
    complete_task: true
    arguments:
      - name: steps
        description: The steps to describe.
        example: >
          - Preheat the oven to 350 degrees F (175 degrees C)
          - In a large bowl, mix together flour, sugar, eggs, and milk.
          - Pour the mixture into a pie crust.
          - Bake in the preheated oven for 20 minutes, or until a knife inserted into the center comes out clean.
```

**estimate_time.yml**

This tasklet will use the `steps` and `ingredients` variables to estimate the preparation time:

```yaml
agent: You are a talented chef. You are given a list of ingredients and a list of preparation steps. You need to estimate the time it will take to prepare the food.

task: >
  Estimate the time it will take to prepare {{ food }}. 

  The ingredients at your disposal are:

    {{ ingredients }}
 
  The preparation steps are: 
 
    {{ steps }}

using:
  - reasoning

tools:
  - name: estimate_time
    description: "To provide the time it will take to prepare the food:"
    complete_task: true
    arguments:
      - name: preparation_time
        description: The time it will take to prepare the food.
        example: 25 minutes
```

**rewrite_nicely.yml**

Finally, this tasklet will use all the variables defined so far to rewrite the recipe in a more engaging and interesting way:

```yaml
agent: You are a talented copywriter specialized in food recipes and food blogging. You are given with a basic food recipe and you need to rewrite it in a more engaging and interesting way.

task: >
  Rewrite the following recipe in a more engaging and interesting way:

  Recipe for {{ food }} ({{ preparation_time }})

  Ingredients:

    {{ ingredients }}

  Preparation steps:

    {{ steps }}

tools:
  - name: rewrite
    description: "To confirm your version of the recipe:"
    complete_task: true
    arguments:
      - name: recipe
        description: The recipe to rewrite.
        example: >
          # **Epic Lazy Day Pancakes**

# ... snippet ...
```

