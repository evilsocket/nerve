#!/usr/bin/env python3
import re
import subprocess


def print_changelog():
    # print changelog
    current_tag = subprocess.run(
        ["git", "describe", "--tags", "--abbrev=0"], capture_output=True, text=True
    ).stdout.strip()
    if current_tag == "":
        # os.system("git log HEAD --oneline")
        interval = "HEAD"
    else:
        print("current tag: %s" % current_tag)
        interval = "%s..HEAD" % current_tag

    print(
        "CHANGELOG:\n\n%s\n"
        % subprocess.run(["git", "log", interval, "--oneline"], capture_output=True, text=True).stdout.strip()
    )


def get_current_version():
    version_match_re = r'^version\s*=\s*"([^"]+)"$'
    with open("pyproject.toml") as fp:
        manifest = fp.read()

    m = re.findall(version_match_re, manifest, re.MULTILINE)
    if len(m) != 1:
        print("could not parse current version from Cargo.toml")
        quit()

    return m[0]


def patch_version(
    filename="pyproject.toml",
    match_re=r'^version\s*=\s*"([^"]+)"$',
    new_version_str='version = "1.0.0"',
):
    with open(filename) as fp:
        data = fp.read()

    result = re.sub(match_re, new_version_str, data, count=0, flags=re.MULTILINE)
    with open(filename, "w+t") as fp:
        fp.write(result)


print_changelog()

current_ver = get_current_version()
next_ver = input(f"current version is {current_ver}, enter next: ")

to_patch = [
    ("pyproject.toml", r'^version\s*=\s*"([^"]+)"$', f'version = "{next_ver}"'),
    ("nerve/__init__.py", r'^__version__\s*=\s*"([^"]+)"$', f'__version__ = "{next_ver}"'),
]

to_add = []
for filename, match_re, new_version_str in to_patch:
    patch_version(filename, match_re, new_version_str)
    to_add.append(filename)

print("git add %s" % " ".join(to_add))

# commit, push and create new tag
print(f"git commit -m 'releasing version {next_ver}'")
print("git push")
print(f"git tag -a v{next_ver} -m 'releasing v{next_ver}'")
print(f"git push origin v{next_ver}")
print()
print("poetry --build publish")
