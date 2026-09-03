from tokens import Token, TokenKind, Span, KEYWORDS
from errors import LexError
from typing import List


class Lexer:
    def __init__(self, source: str, filename: str = "<input>"):
        self.source = source
        self.filename = filename
        self.pos = 0
        self.line = 1
        self.col = 1
        self.tokens: List[Token] = []

    def peek(self, offset: int = 0) -> str:
        p = self.pos + offset
        if p < len(self.source):
            return self.source[p]
        return "\0"

    def advance(self) -> str:
        ch = self.source[self.pos]
        self.pos += 1
        if ch == "\n":
            self.line += 1
            self.col = 1
        else:
            self.col += 1
        return ch

    def skip_whitespace(self) -> None:
        while self.pos < len(self.source) and self.source[self.pos] in " \t\r\n":
            self.advance()

    def skip_comment(self) -> bool:
        if self.peek() == "/" and self.peek(1) == "/":
            while self.pos < len(self.source) and self.peek() != "\n":
                self.advance()
            return True
        if self.peek() == "/" and self.peek(1) == "*":
            self.advance()
            self.advance()
            depth = 1
            while self.pos < len(self.source) and depth > 0:
                if self.peek() == "/" and self.peek(1) == "*":
                    self.advance()
                    self.advance()
                    depth += 1
                elif self.peek() == "*" and self.peek(1) == "/":
                    self.advance()
                    self.advance()
                    depth -= 1
                else:
                    self.advance()
            return True
        return False

    def read_string(self) -> Token:
        start_line, start_col = self.line, self.col
        self.advance()  # skip opening "
        value = ""
        while self.pos < len(self.source) and self.peek() != '"':
            if self.peek() == "\\":
                self.advance()
                escape = self.advance()
                escape_map = {"n": "\n", "t": "\t", "r": "\r", "0": "\\", "\\": "\\"}
                value += escape_map.get(escape, escape)
            else:
                value += self.advance()
        if self.pos >= len(self.source):
            raise LexError("unterminated string literal", Span(start_line, start_col), self.filename)
        self.advance()  # skip closing "
        return Token(TokenKind.STRING, value, Span(start_line, start_col, len(value)))

    def read_char(self) -> Token:
        start_line, start_col = self.line, self.col
        self.advance()  # skip opening '
        if self.peek() == "\\":
            self.advance()
            escape = self.advance()
            escape_map = {"n": "\n", "t": "\t", "r": "\r", "0": "\\", "\\": "\\"}
            ch = escape_map.get(escape, escape)
        else:
            ch = self.advance()
        if self.peek() != "'":
            raise LexError("unterminated character literal", Span(start_line, start_col), self.filename)
        self.advance()  # skip closing '
        return Token(TokenKind.CHAR, ch, Span(start_line, start_col, len(ch)))

    def read_number(self) -> Token:
        start_line, start_col = self.line, self.col
        value = ""
        is_float = False

        if self.peek() == "0" and self.peek(1) in "xX":
            value += self.advance()
            value += self.advance()
            while self.pos < len(self.source) and self.peek() in "0123456789abcdefABCDEF_":
                value += self.advance()
            return Token(TokenKind.INT, value, Span(start_line, start_col, len(value)))

        if self.peek() == "0" and self.peek(1) in "bB":
            value += self.advance()
            value += self.advance()
            while self.pos < len(self.source) and self.peek() in "01_":
                value += self.advance()
            return Token(TokenKind.INT, value, Span(start_line, start_col, len(value)))

        if self.peek() == "0" and self.peek(1) in "oO":
            value += self.advance()
            value += self.advance()
            while self.pos < len(self.source) and self.peek() in "01234567_":
                value += self.advance()
            return Token(TokenKind.INT, value, Span(start_line, start_col, len(value)))

        while self.pos < len(self.source) and self.peek().isdigit():
            value += self.advance()

        if self.peek() == "_" or (self.peek().isdigit()):
            while self.pos < len(self.source) and (self.peek().isdigit() or self.peek() == "_"):
                value += self.advance()

        if self.peek() == ".":
            is_float = True
            value += self.advance()
            while self.pos < len(self.source) and (self.peek().isdigit() or self.peek() == "_"):
                value += self.advance()

        if self.peek() in "eE":
            is_float = True
            value += self.advance()
            if self.peek() in "+-":
                value += self.advance()
            while self.pos < len(self.source) and (self.peek().isdigit() or self.peek() == "_"):
                value += self.advance()

        kind = TokenKind.FLOAT if is_float else TokenKind.INT
        return Token(kind, value, Span(start_line, start_col, len(value)))

    def read_ident(self) -> Token:
        start_line, start_col = self.line, self.col
        value = ""
        while self.pos < len(self.source) and (self.peek().isalnum() or self.peek() == "_"):
            value += self.advance()

        kind = KEYWORDS.get(value, TokenKind.IDENT)
        return Token(kind, value, Span(start_line, start_col, len(value)))

    def read_token(self) -> Token:
        self.skip_whitespace()

        while self.pos < len(self.source):
            if not self.skip_comment():
                break
            self.skip_whitespace()

        if self.pos >= len(self.source):
            return Token(TokenKind.EOF, "", Span(self.line, self.col))

        start_line, start_col = self.line, self.col
        ch = self.peek()

        if ch == '"':
            return self.read_string()
        if ch == "'":
            return self.read_char()
        if ch.isdigit():
            return self.read_number()
        if ch.isalpha() or ch == "_":
            return self.read_ident()

        self.advance()

        single = {
            "(": TokenKind.LPAREN,
            ")": TokenKind.RPAREN,
            "{": TokenKind.LBRACE,
            "}": TokenKind.RBRACE,
            "[": TokenKind.LBRACKET,
            "]": TokenKind.RBRACKET,
            ";": TokenKind.SEMICOLON,
            ",": TokenKind.COMMA,
            ".": TokenKind.DOT,
            "|": TokenKind.PIPE,
            "_": TokenKind.UNDERSCORE,
        }

        if ch in single:
            return Token(single[ch], ch, Span(start_line, start_col))

        if ch == ":":
            if self.peek(0) == ":":
                self.advance()
                return Token(TokenKind.DOUBLE_COLON, "::", Span(start_line, start_col, 2))
            return Token(TokenKind.COLON, ":", Span(start_line, start_col))

        if ch == "=":
            if self.peek(0) == ">":
                self.advance()
                return Token(TokenKind.FAT_ARROW, "=>", Span(start_line, start_col, 2))
            if self.peek(0) == "=":
                self.advance()
                return Token(TokenKind.EQ, "==", Span(start_line, start_col, 2))
            return Token(TokenKind.ASSIGN, "=", Span(start_line, start_col))

        if ch == "!":
            if self.peek(0) == "=":
                self.advance()
                return Token(TokenKind.NEQ, "!=", Span(start_line, start_col, 2))
            return Token(TokenKind.BANG, "!", Span(start_line, start_col))

        if ch == "<":
            if self.peek(0) == "=":
                self.advance()
                return Token(TokenKind.LTE, "<=", Span(start_line, start_col, 2))
            return Token(TokenKind.LT, "<", Span(start_line, start_col))

        if ch == ">":
            if self.peek(0) == "=":
                self.advance()
                return Token(TokenKind.GTE, ">=", Span(start_line, start_col, 2))
            return Token(TokenKind.GT, ">", Span(start_line, start_col))

        if ch == "+":
            if self.peek(0) == "=":
                self.advance()
                return Token(TokenKind.PLUS_ASSIGN, "+=", Span(start_line, start_col, 2))
            return Token(TokenKind.PLUS, "+", Span(start_line, start_col))

        if ch == "-":
            if self.peek(0) == "=":
                self.advance()
                return Token(TokenKind.MINUS_ASSIGN, "-=", Span(start_line, start_col, 2))
            if self.peek(0) == ">":
                self.advance()
                return Token(TokenKind.ARROW, "->", Span(start_line, start_col, 2))
            return Token(TokenKind.MINUS, "-", Span(start_line, start_col))

        if ch == "*":
            if self.peek(0) == "=":
                self.advance()
                return Token(TokenKind.STAR_ASSIGN, "*=", Span(start_line, start_col, 2))
            return Token(TokenKind.STAR, "*", Span(start_line, start_col))

        if ch == "/":
            if self.peek(0) == "=":
                self.advance()
                return Token(TokenKind.SLASH_ASSIGN, "/=", Span(start_line, start_col, 2))
            return Token(TokenKind.SLASH, "/", Span(start_line, start_col))

        if ch == "%":
            if self.peek(0) == "=":
                self.advance()
                return Token(TokenKind.PERCENT_ASSIGN, "%=", Span(start_line, start_col, 2))
            return Token(TokenKind.PERCENT, "%", Span(start_line, start_col))

        if ch == "&":
            if self.peek(0) == "&":
                self.advance()
                return Token(TokenKind.AND, "&&", Span(start_line, start_col, 2))
            return Token(TokenKind.AMP, "&", Span(start_line, start_col))

        if ch == "?":
            return Token(TokenKind.QUESTION, "?", Span(start_line, start_col))

        raise LexError(f"unexpected character: {ch!r}", Span(start_line, start_col), self.filename)

    def tokenize(self) -> List[Token]:
        while True:
            tok = self.read_token()
            self.tokens.append(tok)
            if tok.kind == TokenKind.EOF:
                break
        return self.tokens
