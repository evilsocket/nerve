"""
Simulates the reasoning process at runtime.
"""

from typing import Annotated

import nerve.runtime.state as state

# for docs
EMOJI = "🧠"


def think(thought: Annotated[str, "A thought to think about"]) -> None:
    """
    Adhere strictly to this reasoning framework, ensuring thoroughness, precision, and logical rigor.

    ## Problem Decomposition

    Break the query into discrete, sequential steps.
    Explicitly state assumptions and context.

    ## Stepwise Analysis

    Address each step individually.
    Explain the rationale, principles, or rules applied (e.g., mathematical laws, linguistic conventions).
    Use examples, analogies, or intermediate calculations to illustrate reasoning.

    ## Validation & Error Checking

    Verify logical consistency at each step.
    Flag potential oversights, contradictions, or edge cases.
    Confirm numerical accuracy (e.g., recompute calculations).

    ## Synthesis & Conclusion

    Integrate validated steps into a coherent solution.
    Summarize key insights and ensure the conclusion directly addresses the original query.
    """
    state.append_to_knowledge("thoughts", thought)


def clear_thoughts() -> None:
    """If the reasoning process proved wrong, inconsistent or ineffective, clear your thoughts and start again."""
    state.clear_knowledge("thoughts")
