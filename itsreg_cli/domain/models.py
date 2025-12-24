from __future__ import annotations

from typing import List, Literal

from pydantic import BaseModel


class Message(BaseModel):
    text: str


class Edge(BaseModel):
    predicate: str | dict
    to: int | None = None
    operation: Literal["noop", "save", "append"] | None = None


class Node(BaseModel):
    state: int
    title: str
    messages: List[Message]
    edges: List[Edge] = []
    options: List[str] = []


class Script(BaseModel):
    nodes: List[Node] = []
    entries: List[dict] = []


class Bot(BaseModel):
    id: str
    token: str | None = None
    enabled: bool
    status: Literal["running", "idle", "dead"] | None = "idle"
    script: Script | None = None
