## Changelog

### New Features 🚀
- ✨ Tools can now be directly called from the prompt by interpolation
- ✨ Added new "nerve namespaces" command
- ✨ Introduced inquire namespace to let the agent interactively ask questions to the user in a structured way
- ✨ "nerve create" will now ask to start the agent after its creation

### Bug Fixes 🔧
- Fixed reduced log verbosity when tool returns non-string or dictionary value
- Fixed anytool.create_tool code parameter description
- Fixed extra namespaces are now correctly reported as such in the documentation
- Fixed changelog tooling
- Fixed async wrapper implementation for tool calls

### Chores and Refactoring 🧹
- Refactored each CLI command into its own module
- Refactored generation class

### Miscellaneous
- Various small fixes and general refactoring
