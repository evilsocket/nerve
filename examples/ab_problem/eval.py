#!/usr/bin/env python3
import json
import sys

# evaluation ported from https://github.com/VictorTaelin/ab_challenge_eval/blob/main/main.mjs

HA = "#A"
HB = "#B"
AH = "A#"
BH = "B#"

TOKENS = [HA, HB, AH, BH]


def reduce(xs):
    ys = []
    rwts = 0
    tot_len = len(xs)
    i = 0

    while i < tot_len:
        next = None if i == tot_len - 1 else xs[i + 1]

        if xs[i] == AH and next == HB:
            # A# #B ... becomes ... #B A#
            ys.append(HB)
            ys.append(AH)
            i += 2
            rwts += 1
        elif xs[i] == BH and next == HA:
            # B# #A ... becomes ... #A B#
            ys.append(HA)
            ys.append(BH)
            i += 2
            rwts += 1
        elif xs[i] == AH and next == HA:
            # A# #A ... becomes ... nothing
            i += 2
            rwts += 1
        elif xs[i] == BH and next == HB:
            # B# #B ... becomes ... nothing
            i += 2
            rwts += 1
        else:
            ys.append(xs[i])
            i += 1

    return [ys, rwts]


def solve(xs):
    steps = 0
    term = xs
    work = True

    while True:
        term, work = reduce(term)
        if work > 0:
            steps += work
        else:
            break

    return [term, steps]


def get_solution(message):
    if (
        message["type"] == "agent"
        and "tool_call" in message["data"]
        and message["data"]["tool_call"] is not None
        and message["data"]["tool_call"]["tool_name"] == "solution"
    ):
        return message["data"]["tool_call"]["argument"].strip().split(" ")

    return None


if __name__ == "__main__":
    raw = sys.stdin.read()
    state = json.loads(raw)

    program = state["globals"]["program"].strip().split(" ")
    if any(token not in TOKENS for token in program):
        print("Invalid program")
        exit(1)

    actual, _ = solve(program)
    solution = None

    # find the most recent solution in the chat history
    for message in reversed(state["chat"]["history"]["messages"]):
        solution = get_solution(message)
        if solution is not None:
            if actual == solution:
                # exit code 42 is a special exit code that indicates the solution is correct
                exit(42)
            elif any(token not in TOKENS for token in solution):
                # anything that goes to stdout will be added to the chat history, as feedback to the model
                print(
                    "Invalid solution, provide a single string of tokens separated by spaceswith the solution"
                )
            else:
                print("Solution is incorrect")
            break

    if solution is None:
        print("No solution provided")
