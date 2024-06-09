#!/usr/bin/env python3
import sys
import subprocess


# TODO: read from command line or whatever
SSH_USER_HOST = "zippo@zippo.local"

command = sys.argv[1]

print(f"# {command}")

cmd = [
    "/usr/bin/ssh",
    SSH_USER_HOST,
    sys.argv[1]
]

result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

output = result.stdout.decode('utf-8').strip()
error = result.stderr.decode('utf-8').strip()

if error != '':
    print(error, file=sys.stderr)

if output != '':
    print(output)
