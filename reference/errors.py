from dataclasses import dataclass
from tokens import Span


@dataclass
class CompileError(Exception):
    message: str
    span: Span | None = None
    file: str = "<input>"

    def __str__(self) -> str:
        loc = f"{self.file}:{self.span.line}:{self.span.col}" if self.span else self.file
        return f"error: {self.message}\n  --> {loc}"

    def __repr__(self) -> str:
        return self.__str__()


@dataclass
class LexError(CompileError):
    pass


@dataclass
class ParseError(CompileError):
    pass


@dataclass
class ResolveError(CompileError):
    pass


@dataclass
class TypeError(CompileError):
    pass


@dataclass
class CodegenError(CompileError):
    pass


@dataclass
class RuntimeError_(CompileError):
    pass
