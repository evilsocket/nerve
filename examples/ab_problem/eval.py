#!/usr/bin/env python3
import json
import sys

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


def get_solution_from_message(message):
    if (
        message["type"] == "agent"
        and "tool_call" in message["data"]
        and "argument" in message["data"]["tool_call"]
        and message["data"]["tool_call"] is not None
        and message["data"]["tool_call"]["tool_name"] == "provide_solution"
    ):
        return message["data"]["tool_call"]["argument"].strip().split(" ")

    return None


def get_solution_from_conversation(state):
    for message in reversed(state["chat"]["history"]["messages"]):
        solution = get_solution_from_message(message)
        if solution is not None:
            return solution

    return None


def find_subsequence(arr, target_tuple):
    target = list(target_tuple)
    target_len = len(target)

    for i in range(len(arr) - target_len + 1):
        if arr[i : i + target_len] == target:
            return i
    return None


def solve(program):
    while True:
        rule_found = False
        # find first matching rule
        for source, target in RULES:
            idx = find_subsequence(program, source)
            if idx is not None:
                rule_found = True
                # reduce program by applying substitution
                if target is None:
                    program = program[:idx] + program[idx + len(source) :]
                else:
                    program = (
                        program[:idx] + list(target) + program[idx + len(source) :]
                    )
                break

        # if program can't be further reduced
        if not rule_found:
            break

    return program


if __name__ == "__main__":
    raw = sys.stdin.read()
    state = json.loads(raw)

    raw_program_string = state["globals"]["program"].strip()
    program = raw_program_string.split(" ")
    if any(token not in TOKENS for token in program):
        print("Invalid program")
        exit(1)

    solution = solve(program)

    with open("spoilers.txt", "w+t") as f:
        f.write(f"program: {raw_program_string}\n")
        f.write(f"solution: {solution}\n")

    agent_solution = get_solution_from_conversation(state)

    if agent_solution is None:
        print("No solution provided")
    elif any(token not in TOKENS for token in agent_solution):
        # anything that goes to stdout will be added to the chat history, as feedback to the model
        print(
            f"{agent_solution} is an invalid solution: provide a single string of tokens separated by spaces with the solution"
        )
    elif solution == agent_solution:
        # exit code 42 is a special exit code that indicates the solution is correct
        exit(42)
    else:
        print("Solution is incorrect")
