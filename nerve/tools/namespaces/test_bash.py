import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from nerve.tools.namespaces import bash


class TestBash(unittest.TestCase):
    def test_execute_bash_command(self) -> None:
        # Test basic command execution
        result = bash.execute_bash_command("echo 'hello world'")
        self.assertEqual(result.strip(), "hello world")

    def test_execute_bash_command_with_pipes(self) -> None:
        # Test command with pipes
        result = bash.execute_bash_command("echo 'test' | tr 'e' 'E'")
        self.assertEqual(result.strip(), "tEst")

    def test_execute_bash_command_with_environment_variables(self) -> None:
        # Test command with environment variables
        result = bash.execute_bash_command("TEST_VAR='value' && echo $TEST_VAR")
        self.assertEqual(result.strip(), "value")

    def test_execute_bash_command_with_file_operations(self) -> None:
        # Test command with file operations
        with tempfile.TemporaryDirectory() as temp_dir:
            test_file = Path(temp_dir) / "test.txt"

            # Create a file and write to it
            bash.execute_bash_command(f"echo 'test content' > {test_file}")

            # Read the file
            self.assertTrue(test_file.exists())
            self.assertEqual(test_file.read_text().strip(), "test content")

    @patch("subprocess.check_output")
    def test_execute_bash_command_calls_subprocess(self, mock_check_output: unittest.mock.Mock) -> None:
        # Test that the function calls subprocess.check_output with the right arguments
        mock_check_output.return_value = b"mocked output"

        result = bash.execute_bash_command("some command")

        mock_check_output.assert_called_once_with("some command", shell=True)
        self.assertEqual(result, "mocked output")

    def test_execute_bash_command_error(self) -> None:
        # Test that the function raises an exception for invalid commands
        with self.assertRaises(subprocess.CalledProcessError):
            bash.execute_bash_command("command_that_does_not_exist")
