import asyncio
import pathlib
import pkgutil
import typing as t

import inquirer  # type: ignore
import typer
from pydantic_yaml import to_yaml_str

import nerve
from nerve.cli.defaults import (
    DEFAULT_AGENT_PATH,
    DEFAULT_AGENT_SYSTEM_PROMPT,
    DEFAULT_AGENT_TASK,
    DEFAULT_AGENT_TOOLS,
    DEFAULT_PROMPTS_LOAD_PATH,
)
from nerve.models import Configuration, Tool

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    help="Guided procedure for creating a new agent.",
)
def create(
    path: t.Annotated[
        pathlib.Path,
        typer.Argument(help="Path to the agent or workflow to create"),
    ] = DEFAULT_AGENT_PATH,
    task: t.Annotated[
        str | None,
        typer.Option("--task", "-t", help="Task to create the agent for"),
    ] = None,
    default: t.Annotated[
        bool,
        typer.Option("--default", "-d", help="Use default values."),
    ] = False,
) -> None:
    print(f"🧠 nerve v{nerve.__version__}")

    asyncio.run(create_agent(path.absolute(), task=task, default=default))


def _get_available_namespaces(defaults: list[str]) -> tuple[list[str], list[str]]:
    import nerve.tools.namespaces as namespaces

    available_namespaces = []
    default_entries = []

    for _, modname, _ in pkgutil.iter_modules(namespaces.__path__):
        if modname[0] != "_" and "test" not in modname:
            try:
                module = __import__(f"nerve.tools.namespaces.{modname}", fromlist=[""])
                doc = module.__doc__ or ""
                entry = f"{modname} - {doc.strip()}"
                available_namespaces.append(entry)
                if modname in defaults:
                    default_entries.append(entry)
            except Exception:
                pass

    return available_namespaces, default_entries


def _resolve_system_prompt(system_prompt: str) -> str:
    if system_prompt.startswith("@"):
        system_prompt_file = DEFAULT_PROMPTS_LOAD_PATH / (system_prompt[1:] + ".md")
        if system_prompt_file.exists():
            print(f"🔍 loading system prompt from {system_prompt_file}")
            with open(system_prompt_file) as f:
                system_prompt = f.read()
        else:
            system_prompt_nested = DEFAULT_PROMPTS_LOAD_PATH / system_prompt[1:] / "system.md"
            if system_prompt_nested.exists():
                print(f"🔍 loading system prompt from {system_prompt_nested}")
                with open(system_prompt_nested) as f:
                    system_prompt = f.read()

    return system_prompt.strip()


def _collect_user_prompts() -> list[str]:
    prompts: list[str] = []

    if not DEFAULT_PROMPTS_LOAD_PATH.exists():
        return prompts

    for item in DEFAULT_PROMPTS_LOAD_PATH.iterdir():
        if item.is_dir():
            system_file = item.joinpath("system.md")
            if system_file.exists():
                prompts.append(f"@{item.name}")
        elif item.is_file() and item.suffix == ".md" and item.name != "system.md":
            prompts.append(f"@{item.stem}")

    return sorted(prompts)


# TODO: create a doc page for this.


async def create_agent(path: pathlib.Path, task: str | None = None, default: bool = False) -> None:
    if path.exists():
        print(f"❌ {path} already exists.")
        exit(1)

    available_namespaces, defaults = _get_available_namespaces(DEFAULT_AGENT_TOOLS)
    user_prompts = _collect_user_prompts()

    if default:
        answers = {
            "path": path,
            "system_prompt": DEFAULT_AGENT_SYSTEM_PROMPT,
            "prompt": task or DEFAULT_AGENT_TASK.replace("{{", "{").replace("}}", "}"),
            "tools": DEFAULT_AGENT_TOOLS,
        }
    else:
        print()
        basic_system_prompt_prompt = inquirer.Text(
            "system_prompt", message="System prompt", default=DEFAULT_AGENT_SYSTEM_PROMPT
        )
        questions = [
            inquirer.Path("path", message="Path", default=str(path)),
            basic_system_prompt_prompt
            if not user_prompts
            else inquirer.List(
                "system_prompt",
                message="System prompt",
                choices=["use custom, or:"] + user_prompts,
            ),
            inquirer.Text("prompt", message="Task", default=task or DEFAULT_AGENT_TASK),
            inquirer.Checkbox(
                "tools",
                message="Built-in tools",
                choices=available_namespaces,
                carousel=True,
                default=defaults,
            ),
        ]

        answers = inquirer.prompt(questions)
        answers["tools"] = [tool.split(" - ")[0] for tool in answers["tools"]]  # type: ignore

        if user_prompts:
            answer = str(answers["system_prompt"])
            if answer == "use custom, or:":
                a = inquirer.prompt([basic_system_prompt_prompt])
                answers["system_prompt"] = _resolve_system_prompt(str(a["system_prompt"]))
            else:
                answers["system_prompt"] = _resolve_system_prompt(answer)
        else:
            answers["system_prompt"] = _resolve_system_prompt(str(answers["system_prompt"]))

    example_tool = Tool(
        name="get_weather",
        description="Get the current weather in a given place.",
        arguments=[Tool.Argument(name="place", description="The place to get the weather of.", example="Rome")],
        tool="curl wttr.in/{{ place }}",
    )

    config = Configuration(
        agent=answers["system_prompt"],  # type: ignore
        task=answers["prompt"],  # type: ignore
        using=answers["tools"],  # type: ignore
    )

    config_with_tool = Configuration(
        tools=[example_tool],
    )

    example_tool_lines = ["# You can add custom tools like this:", "#"] + [
        f"# {line}"
        for line in to_yaml_str(config_with_tool, exclude_defaults=True, default_flow_style=False).split("\n")
        if line.strip() != ""
    ]

    agent_code = (
        to_yaml_str(config, exclude_defaults=True, default_flow_style=False) + "\n" + "\n".join(example_tool_lines)
    )

    path = pathlib.Path(str(answers["path"]))
    if path.suffix == "":
        if not path.exists():
            path.mkdir(parents=True)
        path = path / "agent.yml"

    with open(path, "w") as f:
        f.write(agent_code)

    print(f"🤖 agent saved to {path}")

    print()
    answers = inquirer.prompt([inquirer.Confirm("start", message="Start the agent now?", default=True)])
    if answers["start"]:
        import os

        os.system(f"nerve run {path}")
