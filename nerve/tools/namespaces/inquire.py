"""
Let the agent interactively ask questions to the user in a structured way.
"""

from typing import Annotated

import inquirer  # type: ignore
from pydantic import Field

# for docs
EMOJI = "💬"


def ask_question(
    question: Annotated[
        str,
        Field(description="The question to ask the user.", examples=["What is your name?", "What is your age?"]),
    ],
) -> str:
    """Ask a question to the user."""

    print()
    return str(inquirer.prompt([inquirer.Text("question", message=question)])["question"]).strip()


def ask_for_confirmation(
    question: Annotated[
        str,
        Field(
            description="The question to ask the user.", examples=["Do you want to continue?", "Do you want to quit?"]
        ),
    ],
    default: Annotated[bool, Field(description="The default answer to the question.", examples=[True, False])] = False,
) -> str:
    """Ask a confirmation question to the user."""

    print()
    return (
        "YES" if inquirer.prompt([inquirer.Confirm("confirm", message=question, default=default)])["confirm"] else "NO"
    )


def ask_for_single_choice(
    question: Annotated[
        str,
        Field(
            description="The question to ask the user.",
            examples=["What is your favorite color?", "What is your favorite food?"],
        ),
    ],
    choices: Annotated[
        list[str],
        Field(description="The choices to offer the user.", examples=["red", "blue", "green"]),
    ],
) -> str:
    """Ask a single choice question to the user."""

    print()
    return str(inquirer.prompt([inquirer.List("choice", message=question, choices=choices)])["choice"]).strip()


def ask_for_multiple_choice(
    question: Annotated[
        str,
        Field(
            description="The question to ask the user.",
            examples=["What are your favorite colors?", "What are your favorite foods?"],
        ),
    ],
    choices: Annotated[
        list[str],
        Field(description="The choices to offer the user.", examples=["red", "blue", "green"]),
    ],
) -> str:
    """Ask a multiple choice question to the user."""

    print()
    return ", ".join(
        str(choice).strip()
        for choice in inquirer.prompt([inquirer.Checkbox("choices", message=question, choices=choices)])["choices"]
    )
