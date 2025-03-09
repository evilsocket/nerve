import pathlib
import pkgutil

import inquirer  # type: ignore
from pydantic_yaml import to_yaml_str

from nerve.cli.defaults import DEFAULT_AGENT_SYSTEM_PROMPT, DEFAULT_AGENT_TASK, DEFAULT_AGENT_TOOLS
from nerve.models import Configuration, Tool


def _get_available_namespaces(defaults: list[str]) -> tuple[list[str], list[str]]:
    import nerve.tools.namespaces as namespaces

    available_namespaces = []
    default_entries = []

    for _, modname, _ in pkgutil.iter_modules(namespaces.__path__):
        if modname[0] != "_" and not modname.startswith("test_"):
            module = __import__(f"nerve.tools.namespaces.{modname}", fromlist=[""])
            doc = module.__doc__ or ""
            entry = f"{modname} - {doc.strip()}"
            available_namespaces.append(entry)
            if modname in defaults:
                default_entries.append(entry)

    return available_namespaces, default_entries


async def create_agent(path: pathlib.Path, default: bool) -> None:
    if path.exists():
        print(f"❌ {path} already exists.")
        exit(1)

    available_namespaces, defaults = _get_available_namespaces(DEFAULT_AGENT_TOOLS)

    if default:
        answers = {
            "path": path,
            "system_prompt": DEFAULT_AGENT_SYSTEM_PROMPT,
            "prompt": DEFAULT_AGENT_TASK.replace("{{", "{").replace("}}", "}"),
            "tools": DEFAULT_AGENT_TOOLS,
        }
    else:
        print()
        questions = [
            inquirer.Path("path", message="Path", default=str(path)),
            # TODO: maybe implement search in some nice sytem prompt database?
            inquirer.Text("system_prompt", message="System prompt", default=DEFAULT_AGENT_SYSTEM_PROMPT),
            inquirer.Text("prompt", message="Task", default=DEFAULT_AGENT_TASK),
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
