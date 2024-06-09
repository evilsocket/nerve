#!/usr/bin/env python3
import subprocess
import sys

STATE_COMPLETE_EXIT_CODE = 65

# print("  trying '%s' ... " % sys.argv[1], end='', flush=True)

test_binary = './test_binary/test_binary'
cmd = [
    "/usr/bin/lldb",
    "-w",
    "-O", "settings set stop-disassembly-count 20",
    "-o", "run",
    "-o", "q",
    test_binary,
    sys.argv[1]
]

result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

output = result.stdout.decode('utf-8')
error = result.stderr.decode('utf-8')

if error != '':
    raise Exception(error)

if 'stop reason' in output:
    print("CRASH DETECTED\n")
    print(output)
    print()
    sys.exit(STATE_COMPLETE_EXIT_CODE)
