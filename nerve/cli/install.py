import os
import pathlib
import shutil
import tempfile
import typing as t
import zipfile

import requests
import typer

import nerve
from nerve.defaults import (
    DEFAULT_AGENTS_LOAD_PATH,
)
from nerve.models import Configuration, Workflow

cli = typer.Typer(
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
)


def _download_and_unzip(url: str) -> pathlib.Path:
    temp_dir = pathlib.Path(tempfile.mkdtemp())
    local_zip_path = pathlib.Path(os.path.join(temp_dir, "archive.zip"))

    print(f"🌐 downloading {url} ...")

    # download to temporary file
    with requests.get(url, stream=True, verify=True, allow_redirects=True) as response:
        response.raise_for_status()
        with open(local_zip_path, "wb") as zip_file:
            for chunk in response.iter_content(chunk_size=8192):
                zip_file.write(chunk)

    # unzip to temporary directory
    try:
        with zipfile.ZipFile(local_zip_path, "r") as zf:
            for member in zf.infolist():
                file_path = os.path.realpath(os.path.join(temp_dir, member.filename))
                if file_path.startswith(os.path.realpath(temp_dir)):
                    zf.extract(member, temp_dir)
                else:
                    raise Exception("Attempted Path Traversal Attack Detected")

    finally:
        # always remove the zip file
        if local_zip_path.exists():
            os.remove(local_zip_path)

    # in the case of a github repo we expect a single folder named username-repo-commit
    elems = list(temp_dir.iterdir())
    if len(elems) == 1 and elems[0].is_dir():
        temp_dir = elems[0]

    return temp_dir


def _get_source_path_type(source_path: pathlib.Path) -> str | None:
    if Configuration.is_agent_config(source_path):
        return "agent"
    elif Workflow.is_workflow(source_path):
        return "workflow"
    else:
        return None


def _install_from_path(source_path: pathlib.Path, target_path: pathlib.Path, overwrite: bool = False) -> None:
    if not source_path.exists():
        print(f"❌ {source_path} does not exist")
        exit(1)

    if not source_path.is_dir():
        print(f"❌ {source_path} is not a directory")
        exit(1)

    source_path_type = _get_source_path_type(source_path)
    if source_path_type is None:
        print(f"❌ {source_path} is not a valid agent or workflow")
        exit(1)

    target_agent_path = target_path / source_path.name
    if target_agent_path.exists():
        if not overwrite:
            if input(f"⚠️  {target_agent_path} already exists, do you want to overwrite it? [y/N] ") != "y":
                print("❌ installation aborted")
                exit(1)

        shutil.rmtree(target_agent_path)

    target_path.mkdir(parents=True, exist_ok=True)

    shutil.copytree(source_path, target_agent_path)

    print(f"✅ {source_path_type} installed to {target_agent_path}, use `nerve run {source_path.name}` to execute it")


@cli.command(
    context_settings={"help_option_names": ["-h", "--help"]},
    help="Install an agent or workflow from a git repository or local directory.",
)
def install(
    source: t.Annotated[
        str,
        typer.Argument(
            help="Source to install from. It can be a git repository (full url or username/repo) or a local directory."
        ),
    ],
    path: t.Annotated[
        pathlib.Path,
        typer.Option("--path", "-p", help="Path to install to."),
    ] = DEFAULT_AGENTS_LOAD_PATH,
    overwrite: t.Annotated[
        bool,
        typer.Option("--overwrite", "-o", help="Overwrite existing agent or workflow without asking for confirmation."),
    ] = False,
    branch: t.Annotated[
        str,
        typer.Option("--branch", "-b", help="Branch to install from if the source is a git repository."),
    ] = "main",
) -> None:
    print(f"🧠 nerve v{nerve.__version__}")

    source_path_is_tmp = False
    source_path = pathlib.Path(source)

    if not source_path.exists():
        zip_file_url: str = ""
        final_name: str = ""

        if source.startswith("https://github.com"):
            # https://github.com/username/repo (github repository full url)
            zip_file_url = f"{source}/zipball/{branch}"
            final_name = source.split("/")[-1]
        elif source.startswith("https://") and source.endswith(".zip"):
            # https://whatever.dev/agent.zip (zip file url)
            zip_file_url = source
            final_name = source.split("/")[-1].replace(".zip", "")
        elif source.count("/") == 1:
            # username/repo (github repository)
            zip_file_url = f"https://github.com/{source}/zipball/{branch}"
            final_name = source.split("/")[-1]
        else:
            print(f"❌ {source} is not a valid source")
            exit(1)

        try:
            # download, unzip and rename to final name if needed
            temp_dir = _download_and_unzip(zip_file_url)
            if temp_dir.name != final_name:
                new_temp_dir = temp_dir.parent / final_name
                shutil.move(temp_dir, new_temp_dir)
                temp_dir = new_temp_dir
        except Exception as e:
            print(f"❌ error: {e}")
            exit(1)

        source_path_is_tmp = True
        source_path = temp_dir

    try:
        _install_from_path(source_path, path, overwrite)
    except Exception as e:
        print(f"❌ error: {e}")
        exit(1)
    finally:
        if source_path_is_tmp:
            shutil.rmtree(source_path)
