import datetime
import os
import platform
import random
import socket
import string
import time
import typing as t

import requests
from loguru import logger


def _read_clipboard() -> str:
    try:
        import pyperclip
    except ImportError:
        logger.error(
            "pyperclip is not installed. Please install it with `pip install pyperclip` or `pip install nerve-adk[clipboard]`"
        )
        return ""

    val = pyperclip.paste()

    logger.info(f"📋 copied {len(val)} characters from clipboard")
    logger.debug(f"clipboard content: {val}")

    return val


_builtins: dict[str, t.Callable[[], str]] = {
    # date and time
    "CURRENT_DATE": lambda: datetime.datetime.now().strftime("%Y-%m-%d"),
    "CURRENT_TIME": lambda: datetime.datetime.now().strftime("%H:%M:%S"),
    "CURRENT_DATETIME": lambda: datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
    "CURRENT_YEAR": lambda: str(datetime.datetime.now().year),
    "CURRENT_MONTH": lambda: f"{datetime.datetime.now().month:02}",
    "CURRENT_DAY": lambda: f"{datetime.datetime.now().day:02}",
    "CURRENT_WEEKDAY": lambda: datetime.datetime.now().strftime("%A"),
    "TIMEZONE": lambda: datetime.datetime.now().astimezone().tzname() or "UTC",
    "CURRENT_TIMESTAMP": lambda: str(int(time.time())),
    # platform
    "USERNAME": lambda: os.getlogin(),
    "PLATFORM": lambda: platform.system(),
    "OS_VERSION": lambda: platform.version(),
    "ARCHITECTURE": lambda: platform.machine(),
    "PYTHON_VERSION": lambda: platform.python_version(),
    "HOME": lambda: os.path.expanduser("~"),
    "PROCESS_ID": lambda: str(os.getpid()),
    "WORKING_DIR": lambda: os.getcwd(),
    # network
    "LOCAL_IP": lambda: socket.gethostbyname(socket.gethostname()),
    "PUBLIC_IP": lambda: requests.get("https://api.ipify.org?format=json").json()["ip"],
    "HOSTNAME": lambda: socket.gethostname(),
    # misc
    "RANDOM_INT": lambda: str(random.randint(0, 10000)),
    "RANDOM_HEX": lambda: hex(random.getrandbits(64)),
    "RANDOM_FLOAT": lambda: f"{random.uniform(0, 1):.6f}",
    "RANDOM_STRING": lambda: "".join(random.choices(string.ascii_letters + string.digits, k=10)),
    # clipboard
    "CLIPBOARD": _read_clipboard,
}


def is_builtin_variable(variable_name: str) -> bool:
    global _builtins

    return variable_name in _builtins


def get_builtin_variable_value(variable_name: str) -> str:
    global _builtins

    logger.debug(f"getting builtin variable value for {variable_name}")

    return _builtins[variable_name]()
