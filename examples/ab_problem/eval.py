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
        and "data" in message
        and message["data"] is not None
        and message["data"][1] is not None
        and message["data"][1]["action"] == "solution"
        and message["data"][1]["payload"] is not None
    ):
        return message["data"][1]["payload"].strip().split(" ")

    return None


if __name__ == "__main__":
    """
    program = "#A #B #A #A #B A# #B #A #A #A B# B#".strip().split(" ")
    actual, _ = solve(program)
    print(program)
    print(actual)
    exit()
    """
    raw = sys.stdin.read()
    state = json.loads(raw)

    program = state["globals"]["program"].strip().split(" ")
    if any(token not in TOKENS for token in program):
        print("Invalid program")
        exit(1)

    actual, _ = solve(program)
    solution = None

    # find the most recent solution in the chat history
    for message in reversed(state["chat"]["history"]["conversation"]):
        try:
            solution = get_solution(message)
            if solution is not None:
                if actual == solution:
                    exit(42)
                elif any(token not in TOKENS for token in solution):
                    print(
                        "Invalid solution, provide a single string of tokens separated by spaceswith the solution"
                    )
                else:
                    print("Solution is incorrect")
                break
        except Exception as e:
            print(message)
            raise e

    if solution is None:
        print("No solution provided")
