This agent will use git to determine the new commits since the last release in the current folder and generate a markdown changelog.

Run from inside a git repository folder with:

```bash
# use with -q to only print the changelog markdown and disable logs
nerve run examples/changelog -q
```