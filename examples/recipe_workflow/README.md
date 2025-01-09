This is an example workflow for making a food recipe using 4 different agents to:

- Create a list of ingredients (Claude)
- Describe the preparation steps (GPT-4o)
- Estimate the preparation time (GPT-4o-mini)
- Rewrite the recipe in a more engaging and interesting way (GPT-4o)

### Example Usage

```sh
nerve -W recipe_workflow -D food=pizza
```