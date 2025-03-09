This agent will execute SSH commands on a target host in order to perform a given task, run with:

```sh
# you can also use username@server.local
nerve run examples/ssh-agent --task 'determine os version' --host server.local
```