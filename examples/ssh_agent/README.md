Executes tasks on a remote computer by executing bash commands on it via ssh.

### Example Usage

```sh
nerve -G "openai://gpt-4" -T ssh_agent -DSSH_USER_HOST_STRING=username@hostname -P "check which process is consuming more ram"
```