from unittest.mock import MagicMock

import pytest

from nerve.runtime.flow import Flow


def test_flow_singleton() -> None:
    # Create a mock agent
    mock_agent = MagicMock()

    # First flow creation should succeed
    flow1 = Flow(actors=[mock_agent], max_steps=10)
    assert flow1 is not None

    # Second flow creation should raise RuntimeError
    with pytest.raises(RuntimeError) as excinfo:
        flow2 = Flow(actors=[mock_agent], max_steps=10)  # noqa: F841

    # Verify the error message
    assert "A flow is already running" in str(excinfo.value)
