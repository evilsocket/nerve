import sys
import json

"""
An evaluator is a script that receives the current state of the agent via stdin 
and performs some evaluation, at the end of which it can:

1. Exit with a 42 status code if the task is completed successfully.
2. Exit with any other status code if the task is not completed successfully.
3. Return via stdout anything, that'll go to the chat history itself.
"""

if __name__ == "__main__":
    raw = sys.stdin.read()

    # just check for the number 42 in the raw input
    if "42" in raw:
        exit(42)

    # add a feedback message to the chat history
    print("try thinking about a funny book reference to answer")
