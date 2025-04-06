from pathlib import Path

from loguru import logger

from nerve.runtime import state


def is_path_allowed(path_to_check: str, jail: list[str] | None = None) -> bool:
    if not jail:
        return True

    # https://stackoverflow.com/questions/3812849/how-to-check-whether-a-directory-is-a-sub-directory-of-another-directory
    path = Path(path_to_check).resolve().absolute()
    for allowed_path in jail:
        logger.debug(f"interpolating {allowed_path}")
        allowed_path = state.interpolate(allowed_path)
        allowed = Path(allowed_path).resolve().absolute()
        if path == allowed or allowed in path.parents:
            return True

    return False


def path_acl(path_to_check: str, jail: list[str] | None = None) -> None:
    if not is_path_allowed(path_to_check, jail):
        raise ValueError(f"access to path {path_to_check} is not allowed, only allowed paths are: {jail}")


def maybe_text(output: bytes) -> str | bytes:
    try:
        return output.decode("utf-8").strip()
    except UnicodeDecodeError:
        return output
