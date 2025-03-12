import tempfile
import unittest
from pathlib import Path

from nerve.tools.namespaces import shell


class TestShell(unittest.TestCase):
    def test_shell(self) -> None:
        # Test basic command execution
        result = shell.shell("echo 'hello world'")
        self.assertEqual(result.strip(), "hello world")

    def test_shell_with_pipes(self) -> None:
        # Test command with pipes
        result = shell.shell("echo 'test' | tr 'e' 'E'")
        self.assertEqual(result.strip(), "tEst")

    def test_shell_with_environment_variables(self) -> None:
        # Test command with environment variables
        result = shell.shell("TEST_VAR='value' && echo $TEST_VAR")
        self.assertEqual(result.strip(), "value")

    def test_shell_with_file_operations(self) -> None:
        # Test command with file operations
        with tempfile.TemporaryDirectory() as temp_dir:
            test_file = Path(temp_dir) / "test.txt"

            # Create a file and write to it
            shell.shell(f"echo 'test content' > {test_file}")

            # Read the file
            self.assertTrue(test_file.exists())
            self.assertEqual(test_file.read_text().strip(), "test content")

    def test_shell_error(self) -> None:
        app = "foobarbazbiz666"
        output = shell.shell(app)
        self.assertTrue("EXIT CODE: " in output and "ERROR: " in output and "not found" in output and app in output)

    def test_shell_binary_output(self) -> None:
        result = shell.shell("head /bin/sh")
        # verify the result is bytes, not str
        self.assertIsInstance(result, bytes)
        self.assertTrue(result[:4] == b"\x7fELF" or result[:4] == b"\xca\xfe\xba\xbe")
