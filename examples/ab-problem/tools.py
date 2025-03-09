from typing import Annotated

import nerve.runtime.state as state
from loguru import logger

HA = "#A"
HB = "#B"
AH = "A#"
BH = "B#"

TOKENS = [HA, HB, AH, BH]

RULES = (
    # A# #A ... becomes ... nothing
    ((AH, HA), None),
    # A# #B ... becomes ... #B A#
    ((AH, HB), (HB, AH)),
    # B# #A ... becomes ... #A B#
    ((BH, HA), (HA, BH)),
    # B# #B ... becomes ... nothing
    ((BH, HB), None),
)


def _find_subsequence(arr, target_tuple):
    target = list(target_tuple)
    target_len = len(target)

    for i in range(len(arr) - target_len + 1):
        if arr[i : i + target_len] == target:
            return i
    return None


def _solve(program):
    while True:
        rule_found = False
        # find first matching rule
        for source, target in RULES:
            idx = _find_subsequence(program, source)
            if idx is not None:
                rule_found = True
                # reduce program by applying substitution
                if target is None:
                    program = program[:idx] + program[idx + len(source) :]
                else:
                    program = program[:idx] + list(target) + program[idx + len(source) :]
                break

        # if program can't be further reduced
        if not rule_found:
            break

    return program


def provide_solution(solution: Annotated[str, "The solution you found"]) -> str:
    """Use this tool to provide the final solution to the problem."""

    program = state.get_variable("program").split(" ")
    if any(token not in TOKENS for token in program):
        raise ValueError("Invalid program")

    actual_solution = _solve(program)
    agent_solution = solution.strip().split(" ")
    if any(token not in TOKENS for token in agent_solution):
        logger.error(
            f"'{solution}' is an invalid solution: provide a single string of symbols ({', '.join(TOKENS)}) separated by spaces with the solution"
        )
        return f"'{solution}' is an invalid solution: provide a single string of symbols ({', '.join(TOKENS)}) separated by spaces with the solution"
    elif actual_solution == agent_solution:
        logger.info("CORRECT SOLUTION PROVIDED, EVALUATION COMPLETE")
        state.set_task_complete()
        return "CORRECT SOLUTION PROVIDED, EVALUATION COMPLETE"
    else:
        logger.warning(f"incorrect solution | {agent_solution} != {actual_solution}")
        return "Solution is incorrect"
