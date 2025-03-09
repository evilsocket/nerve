import time
import typing as t

from pydantic import BaseModel, Field


class Event(BaseModel):
    timestamp: float = Field(default_factory=time.time)
    name: str
    data: t.Any | None = None
