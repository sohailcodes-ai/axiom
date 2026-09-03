from enum import Enum, auto
from dataclasses import dataclass
from typing import Any


class TokenKind(Enum):
    # Literals
    INT = auto()
    FLOAT = auto()
    STRING = auto()
    CHAR = auto()
    BOOL = auto()
    IDENT = auto()
    UNIT = auto()

    # Keywords
    FN = auto()
    LET = auto()
    MUT = auto()
    STRUCT = auto()
    ENUM = auto()
    IMPL = auto()
    TYPE = auto()
    PUB = auto()
    IF = auto()
    ELSE = auto()
    WHILE = auto()
    FOR = auto()
    IN = auto()
    MATCH = auto()
    RETURN = auto()
    BREAK = auto()
    CONTINUE = auto()
    IMPORT = auto()
    DOMAIN = auto()
    SPAWN = auto()
    AWAIT = auto()
    CHAN = auto()
    TRUE = auto()
    FALSE = auto()
    SELF = auto()
    SUPER = auto()
    PANIC = auto()
    UNREACHABLE = auto()
    MAYBE = auto()
    SOME = auto()
    NONE = auto()
    OK = auto()
    ERR = auto()

    # Operators
    PLUS = auto()
    MINUS = auto()
    STAR = auto()
    SLASH = auto()
    PERCENT = auto()
    EQ = auto()
    NEQ = auto()
    LT = auto()
    GT = auto()
    LTE = auto()
    GTE = auto()
    AND = auto()
    OR = auto()
    NOT = auto()
    ASSIGN = auto()
    PLUS_ASSIGN = auto()
    MINUS_ASSIGN = auto()
    STAR_ASSIGN = auto()
    SLASH_ASSIGN = auto()
    PERCENT_ASSIGN = auto()

    # Delimiters
    LPAREN = auto()
    RPAREN = auto()
    LBRACE = auto()
    RBRACE = auto()
    LBRACKET = auto()
    RBRACKET = auto()
    SEMICOLON = auto()
    COLON = auto()
    DOUBLE_COLON = auto()
    COMMA = auto()
    DOT = auto()
    DOUBLE_DOT = auto()
    ARROW = auto()
    FAT_ARROW = auto()
    AMP = auto()
    AMP_MUT = auto()
    QUESTION = auto()
    PIPE = auto()
    UNDERSCORE = auto()
    BANG = auto()

    # Special
    EOF = auto()
    NEWLINE = auto()


KEYWORDS = {
    "fn": TokenKind.FN,
    "let": TokenKind.LET,
    "mut": TokenKind.MUT,
    "struct": TokenKind.STRUCT,
    "enum": TokenKind.ENUM,
    "impl": TokenKind.IMPL,
    "type": TokenKind.TYPE,
    "pub": TokenKind.PUB,
    "if": TokenKind.IF,
    "else": TokenKind.ELSE,
    "while": TokenKind.WHILE,
    "for": TokenKind.FOR,
    "in": TokenKind.IN,
    "match": TokenKind.MATCH,
    "return": TokenKind.RETURN,
    "break": TokenKind.BREAK,
    "continue": TokenKind.CONTINUE,
    "import": TokenKind.IMPORT,
    "domain": TokenKind.DOMAIN,
    "spawn": TokenKind.SPAWN,
    "await": TokenKind.AWAIT,
    "chan": TokenKind.CHAN,
    "true": TokenKind.TRUE,
    "false": TokenKind.FALSE,
    "self": TokenKind.SELF,
    "super": TokenKind.SUPER,
    "panic": TokenKind.PANIC,
    "unreachable": TokenKind.UNREACHABLE,
    "maybe": TokenKind.MAYBE,
    "Some": TokenKind.SOME,
    "None": TokenKind.NONE,
    "Ok": TokenKind.OK,
    "Err": TokenKind.ERR,
}


@dataclass
class Span:
    line: int
    col: int
    length: int = 1

    def end_col(self) -> int:
        return self.col + self.length


@dataclass
class Token:
    kind: TokenKind
    value: str
    span: Span

    def __repr__(self) -> str:
        if self.value:
            return f"Token({self.kind.name}, {self.value!r}, L{self.span.line}:C{self.span.col})"
        return f"Token({self.kind.name}, L{self.span.line}:C{self.span.col})"
