Fuzzes a test binary in order to find a crash.

**NOTEs**

* The fuzz.py script assumes the presence of LLDB.
* The test_binary must be compiled with make.

### Example Usage

```sh
nerve -G "openai://gpt-4" -T fuzzer
```