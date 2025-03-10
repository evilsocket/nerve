import tempfile
import unittest
from pathlib import Path

from nerve.tools.namespaces import shell


class TestShell(unittest.TestCase):
    def test_execute_shell_command(self) -> None:
        # Test basic command execution
        result = shell.execute_shell_command("echo 'hello world'")
        self.assertEqual(result.strip(), "hello world")

    def test_execute_shell_command_with_pipes(self) -> None:
        # Test command with pipes
        result = shell.execute_shell_command("echo 'test' | tr 'e' 'E'")
        self.assertEqual(result.strip(), "tEst")

    def test_execute_shell_command_with_environment_variables(self) -> None:
        # Test command with environment variables
        result = shell.execute_shell_command("TEST_VAR='value' && echo $TEST_VAR")
        self.assertEqual(result.strip(), "value")

    def test_execute_shell_command_with_file_operations(self) -> None:
        # Test command with file operations
        with tempfile.TemporaryDirectory() as temp_dir:
            test_file = Path(temp_dir) / "test.txt"

            # Create a file and write to it
            shell.execute_shell_command(f"echo 'test content' > {test_file}")

            # Read the file
            self.assertTrue(test_file.exists())
            self.assertEqual(test_file.read_text().strip(), "test content")

    def test_execute_shell_command_error(self) -> None:
        output = shell.execute_shell_command("foobarbazbiz666")
        self.assertEqual(output, "EXIT CODE: 127\nERROR: /bin/sh: foobarbazbiz666: command not found")

    def test_execute_shell_command_binary_output(self) -> None:
        result = shell.execute_shell_command("printf '\\xff\\xfe\\x00\\x01'")
        # verify the result is bytes, not str
        self.assertIsInstance(result, bytes)
        self.assertEqual(result, b"\xff\xfe\x00\x01")
