import os
import tempfile
import unittest
from pathlib import Path

from nerve.tools.utils import is_path_allowed, maybe_text, path_acl


class TestUtils(unittest.TestCase):
    def test_maybe_text_with_valid_utf8(self) -> None:
        # Test with valid UTF-8 string
        input_bytes = b"Hello, world!"
        result = maybe_text(input_bytes)
        self.assertEqual(result, "Hello, world!")
        self.assertIsInstance(result, str)

    def test_maybe_text_with_invalid_utf8(self) -> None:
        # Test with invalid UTF-8 sequence
        input_bytes = bytes([0xFF, 0xFE, 0xFD])  # Invalid UTF-8 sequence
        result = maybe_text(input_bytes)
        self.assertEqual(result, input_bytes)
        self.assertIsInstance(result, bytes)

    def test_maybe_text_with_empty_bytes(self) -> None:
        # Test with empty bytes
        input_bytes = b""
        result = maybe_text(input_bytes)
        self.assertEqual(result, "")
        self.assertIsInstance(result, str)

    def test_maybe_text_with_mixed_content(self) -> None:
        # Test with mixed content (valid UTF-8 + invalid sequence)
        input_bytes = b"Valid text" + bytes([0xFF, 0xFE]) + b"more text"
        result = maybe_text(input_bytes)
        self.assertEqual(result, input_bytes)
        self.assertIsInstance(result, bytes)

    def test_maybe_text_with_non_bytes_input(self) -> None:
        # Test with non-bytes input (should handle gracefully or raise appropriate error)
        with self.assertRaises(AttributeError):
            maybe_text("already a string")  # type: ignore

    def test_path_acl_with_no_jail(self) -> None:
        # Test with empty jail list
        jail: list[str] = []
        path = "/some/random/path"
        self.assertTrue(is_path_allowed(path, jail))

        # Test path_acl with empty jail
        path_acl(path, jail)  # Should not raise an exception with empty jail

    def test_path_acl_with_jail(self) -> None:
        # Test with jail containing paths
        jail = ["/allowed/path", "/another/allowed"]

        # Test allowed path
        allowed_path = "/allowed/path/file.txt"
        self.assertTrue(is_path_allowed(allowed_path, jail))
        path_acl(allowed_path, jail)  # Should not raise an exception

        # Test another allowed path
        another_allowed = "/another/allowed/subdir/file.txt"
        self.assertTrue(is_path_allowed(another_allowed, jail))
        path_acl(another_allowed, jail)  # Should not raise an exception

        # Test disallowed path
        disallowed_path = "/disallowed/path/file.txt"
        self.assertFalse(is_path_allowed(disallowed_path, jail))

        # Test path_acl with disallowed path (should raise ValueError)
        with self.assertRaises(ValueError):
            path_acl(disallowed_path, jail)

    def test_path_acl_with_symlinks(self) -> None:
        # Create temporary directories for testing
        with tempfile.TemporaryDirectory() as allowed_dir, tempfile.TemporaryDirectory() as disallowed_dir:
            allowed_path = Path(allowed_dir)
            disallowed_path = Path(disallowed_dir)

            # Create a symlink inside allowed directory pointing to disallowed directory
            symlink_path = allowed_path / "symlink_to_outside"
            os.symlink(str(disallowed_path), str(symlink_path))

            jail = [str(allowed_path)]

            # Test direct access to allowed path
            self.assertTrue(is_path_allowed(str(allowed_path), jail))

            # Test symlink that resolves outside jail
            symlink_file_path = str(symlink_path / "some_file.txt")
            self.assertFalse(is_path_allowed(symlink_file_path, jail))

            with self.assertRaises(ValueError):
                path_acl(symlink_file_path, jail)
