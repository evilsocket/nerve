# Namespaces

Nerve offers a rich set of predefined tools, organized in namespaces, that the agent can import [with the using directive](tasklets.md#tools). **If a specific model does not natievly support tool calling, Nerve will use its own XML based format to enable tool calling.**

This page contains the list of namespaces available in Nerve, with the descriptive prompt that will be provided to the model.

## Memory

Use these memory-related actions to store and retrieve meaningful information obtained from the output of previous actions. Information worth storing includes anything that could be useful for accomplishing your current task, goal, or plan. If the information is no longer relevant to the current goal, or if it contains errors based on new information, it should be deleted.


* To store a memory with any key: `<save_memory key="my-note">put here the custom data you want to keep for later</save_memory>`
* To delete a memory you previously stored given its key: `<delete_memory key="my-note"/>`

<!-- Is there a reason the second bullet point says "given its key" but the first one doesn't? It looks like both code snippets mention the `key` - just checking! -->

## Time

<!-- Let's add a snippet here about why a user would want to use a time-related action (like you did in the Memory section above). It's best if headers have a bit of context below them before going right into the code/details! -->

* To pause for a given amount of seconds: `<wait>5</wait>`

## Goal

Use this action to update your current goal.

* When a new goal is required: `<update_goal>your new goal</update_goal>`

## Planning

Use these actions to maintain a structured plan for achieving your current goal. This namespace allows you to think the problem through step-by-step. Each step must build upon the steps that precede it, moving methodically towards accomplishing your current goal:

* To add a step to your plan: `<add_plan_step>complete the task</add_plan_step>`
* To remove a step from your plan, given its position: `<delete_plan_step>2</delete_plan_step>`
* To set a step from your plan as completed: `<set_step_completed>2</set_step_completed>`
* To set a step from your plan as incomplete: `<set_step_incomplete>2</set_step_incomplete>`
* You may hit a dead end if your original plan wasn't the right direction, as you'll notice when your efforts fail to bring you closer to your goal. Whenever you reach a dead end, clear your existing plan and start from scratch with a new one by using: `<clear_plan/>`

## Task

Use these actions to set the task as completed.

* When your objective has been reached: `<task_complete>a brief report about why the task is complete</task_complete>`
* If you determine that the given goal or task is impossible given the information you have: `<task_impossible>a brief report about why the task is impossible</task_impossible>`

## Filesystem

You can use the filesystem actions to read files and folders from the disk.

* To read the contents of a file from disk: `<read_file>/path/to/file/to/read</read_file>`
* To list the files in a folder on disk: `<list_folder_contents>/path/to/folder</list_folder_contents>`
* To append a structured JSON object to the log file: `<append_to_file>{
      "title": "Example title",
      "description": "Example description.",
    }</append_to_file>`

## Knowledge

Use these actions to search and retrieve information from your long term storage.

* All information from your long term storage is true. To search for information on your long-term storage: `<search>what is the biggest city in the world?</search>`

## Web

You can use the web actions to execute HTTP requests.

* To add a permanent header to all future requests:
 `<http_set_header name="X-Header">some-value-for-the-header</http_set_header>`
      * This is useful for cookies, session information, and authentication tokens.
* To reset your current session and clear all headers: `<http_clear_headers/>`
* To create and send an HTTP request with the specified method: `<http_request method="GET">/index.php?id=1</http_request>`

## Shell

Use this action to execute shell commands and get their output.

*  `<shell>ls -la</shell>`
