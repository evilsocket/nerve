import base64
import unittest
from unittest.mock import MagicMock, patch

from nerve.tools.compiler import wrap_tool_function


class TestWrapToolFunction(unittest.IsolatedAsyncioTestCase):
    @patch("nerve.tools.compiler.state")
    async def test_exception_in_wrapped_function_returns_error_string(self, _: MagicMock) -> None:
        def test_func() -> None:
            raise ValueError("test error")

        wrapped_func = wrap_tool_function(test_func)
        result = await wrapped_func()

        self.assertTrue(isinstance(result, str))
        self.assertTrue(result.startswith("ERROR in test_func: test error"))

    async def test_no_mime_returns_original_result(self) -> None:
        def test_func() -> str:
            return "test result"

        wrapped_func = wrap_tool_function(test_func)
        result = await wrapped_func()

        self.assertEqual(result, "test result")

    async def test_image_mime_returns_image_dict(self) -> None:
        test_bytes = b"test image data"
        expected_b64 = base64.b64encode(test_bytes).decode("utf-8")

        def test_func() -> bytes:
            return test_bytes

        mime = "image/png"
        wrapped_func = wrap_tool_function(test_func, mime=mime)
        result = await wrapped_func()

        self.assertEqual(result["type"], "image_url")
        self.assertEqual(result["image_url"]["url"], f"data:{mime};base64,{expected_b64}")

    @patch("nerve.tools.compiler.logger")
    @patch("nerve.tools.compiler.exit")
    async def test_invalid_mime_raises_exception(self, mock_exit: MagicMock, mock_logger: MagicMock) -> None:
        def test_func() -> str:
            return "test result"

        wrapped_func = wrap_tool_function(test_func, mime="invalid/mime")

        await wrapped_func()

        mock_logger.error.assert_called_once_with("tool test_func references an unsupported mime type: invalid/mime")
        mock_exit.assert_called_once_with(1)
