import asyncio
import typing as t

import asyncssh
from loguru import logger

# TODO: read from command line
SSH_HOST = 'zippo.local'
SSH_PORT = 22
SSH_USER = 'zippo'
SSH_PASS = 'zippo'
SHELL = '/usr/bin/bash'


def get_task_main_story() -> str:
    return """\
You are a senior developer and computer expert with years of linux experience. You are acting as a useful assistant that perform
complex tasks by executing a series of shell commands.
"""


def get_task_description() -> str:
    # TODO: read from command line if not interactive
    return input('\nenter task description> ')


def validate_task_completion(state) -> bool:
    try:
        logger.success("\n\n%s\n\n" % state.toJSON())
    except Exception as e:
        logger.error(f"can't serialize state: {e}")
    return True


async def create_client() -> t.Optional[str]:
    try:
        conn = await asyncssh.connect(SSH_HOST, SSH_PORT, username=SSH_USER, password=SSH_PASS, known_hosts=None)
        logger.success(f"connected to {SSH_USER}@{SSH_HOST}:{SSH_PORT} ...")
        return conn
    except Exception as e:
        logger.error("failed to authenticate")
        logger.error(str(e))
        return None

# TODO: show terminal and chain of thoughts in seperate panels


async def execute_command(
    client: t.Any, command: str, *, max_output_len: int = 555_000, timeout: int = 300
) -> str:
    print(f"# {command}")

    # gives the model a little help since we don't have a real interactive session
    if 'apt' in command and 'install' in command:
        command += ' -y'

    async with client.create_process(SHELL) as process:  # type: ignore
        process.stdin.write(command + "\n" + "exit" + "\n")
        try:
            stdout_output, stderr_output = await asyncio.wait_for(process.communicate(), timeout=timeout)
        except asyncio.TimeoutError:
            process.terminate()
            stdout_output = ""
            stderr_output = "[command timed out]"

    if stderr_output != '':
        output = f"{stdout_output}\nERROR: {stderr_output}".strip()
    else:
        output = stdout_output

    if not output:
        output = "[command finished with no output]"

    if len(output) > max_output_len:
        output = output[:max_output_len] + "\n[output truncated]"

    print(f"\n{output}\n")

    return str(output).strip()
