import os
import tempfile
import unittest
from pathlib import Path

from nerve.tools.namespaces import filesystem


class TestFilesystem(unittest.TestCase):
    def setUp(self) -> None:
        # Create temporary directory for testing
        self.temp_dir = tempfile.TemporaryDirectory()
        self.test_dir = Path(self.temp_dir.name)

        # Create test files and subdirectories
        self.test_file = self.test_dir / "test_file.txt"
        self.test_file.write_text("test content")

        self.test_subdir = self.test_dir / "subdir"
        self.test_subdir.mkdir()

        self.test_subfile = self.test_subdir / "subfile.txt"
        self.test_subfile.write_text("subfile content")

        # Reset jail between tests
        filesystem.jail = []

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def test_list_folder_contents_no_jail(self) -> None:
        # Test without jail restrictions
        result = filesystem.list_folder_contents(str(self.test_dir))
        self.assertIn("test_file.txt", result)
        self.assertIn("subdir", result)

    def test_read_file_no_jail(self) -> None:
        # Test without jail restrictions
        result = filesystem.read_file(str(self.test_file))
        self.assertEqual(result, "test content")

    def test_list_folder_contents_with_jail_allowed(self) -> None:
        # Set jail to allow only the test directory
        filesystem.jail = [str(self.test_dir)]

        # Should work for the test directory
        result = filesystem.list_folder_contents(str(self.test_dir))
        self.assertIn("test_file.txt", result)

        # Should work for subdirectories
        result = filesystem.list_folder_contents(str(self.test_subdir))
        self.assertIn("subfile.txt", result)

    def test_read_file_with_jail_allowed(self) -> None:
        # Set jail to allow only the test directory
        filesystem.jail = [str(self.test_dir)]

        # Should work for files in the test directory
        result = filesystem.read_file(str(self.test_file))
        self.assertEqual(result, "test content")

        # Should work for files in subdirectories
        result = filesystem.read_file(str(self.test_subfile))
        self.assertEqual(result, "subfile content")

    def test_list_folder_contents_with_jail_denied(self) -> None:
        # Create another temporary directory outside the jail
        with tempfile.TemporaryDirectory() as outside_dir:
            # Set jail to allow only the test directory
            filesystem.jail = [str(self.test_dir)]

            # Should raise ValueError for directories outside the jail
            with self.assertRaises(ValueError):
                filesystem.list_folder_contents(outside_dir)

    def test_read_file_with_jail_denied(self) -> None:
        # Create a file outside the jail
        with tempfile.NamedTemporaryFile(mode="w+") as outside_file:
            outside_file.write("outside content")
            outside_file.flush()

            # Set jail to allow only the test directory
            filesystem.jail = [str(self.test_dir)]

            # Should raise ValueError for files outside the jail
            with self.assertRaises(ValueError):
                filesystem.read_file(outside_file.name)

    def test_multiple_jail_paths(self) -> None:
        # Create another temporary directory
        with tempfile.TemporaryDirectory() as second_dir:
            second_path = Path(second_dir)
            second_file = second_path / "second_file.txt"
            second_file.write_text("second content")

            # Set jail to allow both directories
            filesystem.jail = [str(self.test_dir), second_dir]

            # Should work for both directories
            result1 = filesystem.read_file(str(self.test_file))
            self.assertEqual(result1, "test content")

            result2 = filesystem.read_file(str(second_file))
            self.assertEqual(result2, "second content")

    def test_path_allowed_with_symlinks(self) -> None:
        # Create a directory outside the jail
        with tempfile.TemporaryDirectory() as outside_dir:
            outside_path = Path(outside_dir)
            outside_file = outside_path / "outside_file.txt"
            outside_file.write_text("outside content")

            # Create a symlink inside the jail pointing outside
            symlink_path = self.test_dir / "symlink_to_outside"
            os.symlink(outside_dir, str(symlink_path))

            # Set jail to allow only the test directory
            filesystem.jail = [str(self.test_dir)]

            # Should raise ValueError for symlinks that resolve outside the jail
            with self.assertRaises(ValueError):
                filesystem.list_folder_contents(str(symlink_path))

            with self.assertRaises(ValueError):
                filesystem.read_file(str(symlink_path / "outside_file.txt"))

    def test_read_file_binary(self) -> None:
        # Create a binary file with non-UTF8 content
        binary_file = self.test_dir / "binary_file"
        binary_content = bytes([0xFF, 0xFE, 0xFD])  # Invalid UTF-8 sequence
        binary_file.write_bytes(binary_content)

        # Set jail to allow the test directory
        filesystem.jail = [str(self.test_dir)]

        # Should return bytes for binary content
        result = filesystem.read_file(str(binary_file))
        self.assertIsInstance(result, bytes)
        self.assertEqual(result, binary_content)
