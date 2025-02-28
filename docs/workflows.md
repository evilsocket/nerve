# Workflows

Workflows allow you to orchestrate multiple [tasklets](tasklets.md) as a sequence of steps. Each tasklet can be executed by a different model, if desired, and data is passed between them by setting variables and interpolating them into the prompt of the next tasklet. Each agent will have its own prompt, guidance, and functions, acting as an independent functional unit.

For instance, the [recipe workflow example](https://github.com/evilsocket/nerve/tree/main/examples/recipe_workflow) shows how to orchestrate four different agents to:

- Create a list of ingredients (Claude)
- Describe the preparation steps (GPT-4o)
- Estimate the preparation time (GPT-4o-mini)
- Rewrite the recipe in a more engaging and interesting way (GPT-4o)

The YAML definition of the workflow is:

```yaml
name: "Write a Recipe"
description: "A workflow for writing a recipe."

tasks:
  create_list_of_ingredients:
    generator: anthropic://claude

  describe_preparation_steps:
    generator: openai://gpt-4o

  estimate_time:
    generator: openai://gpt-4o-mini

  rewrite_nicely:
    generator: openai://gpt-4o

# what final variable contains the report of the workflow
report: $report
```

Each element of the `tasks` array is a tasklet that will be executed in the order specified.

**create_list_of_ingredients.yml**

This first agent will create a list of ingredients that will be saved in the `ingredients` variable. Once the tool is executed, the task will be marked as complete and the next tasklet will execute.

```yaml
system_prompt: You are a talented chef.

prompt: prepare a list of ingredients for $food

guidance:
  - Once you have made a list of ingredients, use the create_list_of_ingredients tool to confirm the decision.

tool_box:
  - name: Tools
    tools:
      - name: create_list_of_ingredients
        description: "To provide the ingredients one per line as an organized list:"
        store_to: ingredients
        complete_task: true
        example_payload: >
          - 1 cup of flour
          - 1 cup of sugar
          - 1 cup of eggs
          - 1 cup of milk
          - 1 cup of butter
```

**describe_preparation_steps.yml**

This tasklet will use the `ingredients` variable, by interpolating it in the prompt with the `$` symbol, to describe the preparation steps.

```yaml
system_prompt: You are a talented and creative chef.

# $ingredients used here
prompt: describe the preparation steps for $food. The ingredients at your disposal are $ingredients.

guidance:
  - Use the describe_preparation_steps tool to describe each step in the preparation.
  - Once you have described each step in the preparation of the pie set your task as complete.

tool_box:
  - name: Tools
    tools:
      - name: describe_preparation_steps
        description: "To provide the preparation steps one per line as an organized list:"
        store_to: steps
        complete_task: true
        example_payload: >
          - Preheat the oven to 350 degrees F (175 degrees C)
          - In a large bowl, mix together flour, sugar, eggs, and milk.
          - Pour the mixture into a pie crust.
          - Bake in the preheated oven for 20 minutes, or until a knife inserted into the center comes out clean.
```

**estimate_time.yml**

This tasklet will use the `steps` and `ingredients` variables to estimate the preparation time:

```yaml
system_prompt: You are a talented chef. You are given a list of ingredients and a list of preparation steps. You need to estimate the time it will take to prepare the food.

# $ingredients and $steps used here
prompt: >
  Estimate the time it will take to prepare $food. 

  The ingredients at your disposal are:
    $ingredients
  The preparation steps are: 
    $steps

guidance:
  - Once you have made an estimation, use the estimate_time tool to confirm the decision.

tool_box:
  - name: Tools
    tools:
      - name: estimate_time
        description: "To provide the time it will take to prepare the food:"
        store_to: preparation_time
        complete_task: true
        example_payload: 25 minutes
```

**rewrite_nicely.yml**

Finally, this tasklet will use all the variables defined so far to rewrite the recipe in a more engaging and interesting way:

```yaml
system_prompt: You are a talented copywriter specialized in food recipes and food blogging. You are given with a basic food recipe and you need to rewrite it in a more engaging and interesting way.

# $food, $preparation_time, $ingredients and $steps used here
prompt: >
  Rewrite the following recipe in a more engaging and interesting way:

  Recipe for $food ($preparation_time)

  Ingredients:

    $ingredients

  Preparation steps:

    $steps

guidance:
  - Once you have completed the task, use the rewrite tool to confirm the decision.

tool_box:
  - name: Tools
    tools:
      - name: rewrite
        description: "To confirm your version of the recipe:"
        store_to: report
        complete_task: true
        example_payload: >
          # **Epic Lazy Day Pancakes**

# ... snippet ...
```

