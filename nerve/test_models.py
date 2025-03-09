import pathlib
import tempfile
import unittest

from nerve.models import Configuration


class TestConfiguration(unittest.TestCase):
    def test_is_legacy_with_system_prompt(self) -> None:
        """Test that is_legacy returns True when system_prompt is set"""
        config = Configuration(system_prompt="This is a legacy prompt")
        self.assertTrue(config.is_legacy)

    def test_is_legacy_without_system_prompt(self) -> None:
        """Test that is_legacy returns False when system_prompt is None"""
        config = Configuration(agent="Agent prompt", task="Task description")
        self.assertFalse(config.is_legacy)

    def test_is_legacy_with_empty_config(self) -> None:
        """Test that is_legacy returns False with default configuration"""
        config = Configuration()
        self.assertFalse(config.is_legacy)

    def test_system_prompt_excluded_from_serialization(self) -> None:
        """Test that system_prompt is excluded when serializing the configuration"""
        config = Configuration(system_prompt="This should be excluded", agent="Agent prompt")
        serialized = config.model_dump()
        self.assertNotIn("system_prompt", serialized)

    def test_system_prompt_loaded_from_file(self) -> None:
        """Test that system_prompt is loaded if present in a file"""

        with tempfile.NamedTemporaryFile(suffix=".yml", delete=False) as temp_file:
            temp_file.write(b"system_prompt: Legacy prompt")
            temp_path = pathlib.Path(temp_file.name)

        try:
            loaded_config = Configuration.from_path(temp_path)

            self.assertEqual(loaded_config.system_prompt, "Legacy prompt")
            self.assertTrue(loaded_config.is_legacy)
        finally:
            temp_path.unlink()
