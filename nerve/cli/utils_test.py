import pathlib
from unittest.mock import MagicMock, patch

import pytest
import typer

from nerve.cli.utils import _resolve_input_path
from nerve.defaults import DEFAULT_AGENTS_LOAD_PATH


def test_resolve_input_path_exists() -> None:
    """Test when input_path exists directly."""
    mock_path = MagicMock(spec=pathlib.Path)
    mock_path.exists.return_value = True

    result = _resolve_input_path(mock_path)

    assert result == mock_path
    mock_path.exists.assert_called_once()


def test_resolve_input_path_with_yaml_extension() -> None:
    """Test when input_path with .yml extension exists."""
    mock_path = MagicMock(spec=pathlib.Path)
    mock_path.exists.side_effect = [False, True]
    mock_path_with_yaml = MagicMock(spec=pathlib.Path)
    mock_path.with_suffix.return_value = mock_path_with_yaml
    mock_path_with_yaml.exists.return_value = True

    result = _resolve_input_path(mock_path)

    assert result == mock_path_with_yaml
    mock_path.with_suffix.assert_called_once_with(".yml")


def test_resolve_input_path_in_home_with_yaml() -> None:
    """Test when input_path with .yml extension exists in the default agents directory."""
    mock_path = MagicMock(spec=pathlib.Path)
    mock_path.exists.side_effect = [False, False]
    mock_path.with_suffix.return_value.exists.return_value = False
    mock_path.is_absolute.return_value = False

    in_home_path = MagicMock(spec=pathlib.Path)
    in_home_path.exists.return_value = False

    in_home_yaml_path = MagicMock(spec=pathlib.Path)
    in_home_yaml_path.exists.return_value = True
    in_home_path.with_suffix.return_value = in_home_yaml_path

    with patch("nerve.cli.utils.DEFAULT_AGENTS_LOAD_PATH", DEFAULT_AGENTS_LOAD_PATH):
        with patch("pathlib.Path.__truediv__", return_value=in_home_path):
            result = _resolve_input_path(mock_path)

    assert result == in_home_yaml_path
    in_home_path.with_suffix.assert_called_once_with(".yml")


def test_resolve_input_path_not_exists() -> None:
    """Test when input_path does not exist anywhere, should raise typer.Abort."""
    mock_path = MagicMock(spec=pathlib.Path)
    mock_path.exists.side_effect = [False, False]
    mock_path.with_suffix.return_value.exists.return_value = False
    mock_path.is_absolute.return_value = False

    in_home_path = MagicMock(spec=pathlib.Path)
    in_home_path.exists.return_value = False
    in_home_path.with_suffix.return_value.exists.return_value = False

    with patch("nerve.cli.utils.DEFAULT_AGENTS_LOAD_PATH", DEFAULT_AGENTS_LOAD_PATH):
        with patch("pathlib.Path.__truediv__", return_value=in_home_path):
            with patch("nerve.cli.utils.logger.error") as mock_logger:
                with pytest.raises(typer.Abort):
                    _resolve_input_path(mock_path)

    mock_logger.assert_called_once()
