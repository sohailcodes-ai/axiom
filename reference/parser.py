from tokens import Token, TokenKind, Span
from ast_nodes import *
from errors import ParseError
from typing import List, Optional


class Parser:
    def __init__(self, tokens: List[Token], filename: str = "<input>"):
        self.tokens = tokens
        self.pos = 0
        self.filename = filename

    def peek(self, offset: int = 0) -> Token:
        p = self.pos + offset
        if p < len(self.tokens):
            return self.tokens[p]
        return Token(TokenKind.EOF, "", Span(0, 0))

    def advance(self) -> Token:
        tok = self.tokens[self.pos]
        self.pos += 1
        return tok

    def expect(self, kind: TokenKind) -> Token:
        tok = self.peek()
        if tok.kind != kind:
            raise ParseError(
                f"expected {kind.name}, got {tok.kind.name} ({tok.value!r})",
                tok.span,
                self.filename,
            )
        return self.advance()

    def skip(self, kind: TokenKind) -> bool:
        if self.peek().kind == kind:
            self.advance()
            return True
        return False

    def at(self, *kinds: TokenKind) -> bool:
        return self.peek().kind in kinds

    def at_expr_start(self) -> bool:
        kind = self.peek().kind
        return kind in (
            TokenKind.INT, TokenKind.FLOAT, TokenKind.STRING, TokenKind.CHAR,
            TokenKind.BOOL, TokenKind.IDENT, TokenKind.LPAREN, TokenKind.LBRACE,
            TokenKind.MINUS, TokenKind.BANG, TokenKind.AMP, TokenKind.IF,
            TokenKind.WHILE, TokenKind.FOR, TokenKind.MATCH, TokenKind.RETURN,
            TokenKind.BREAK, TokenKind.CONTINUE, TokenKind.TRUE, TokenKind.FALSE,
            TokenKind.UNIT, TokenKind.SOME, TokenKind.NONE, TokenKind.PANIC,
            TokenKind.UNREACHABLE,
        )

    def parse_program(self) -> Program:
        start = self.peek().span
        decls = []
        while not self.at(TokenKind.EOF):
            decls.append(self.parse_decl())
        end = self.peek().span
        return Program(decls, Span(start.line, start.col))

    def parse_decl(self) -> Decl:
        tok = self.peek()

        if tok.kind == TokenKind.PUB:
            self.advance()
            return self.parse_decl_after_pub(tok.span)

        if tok.kind == TokenKind.STRUCT:
            return self.parse_struct_decl()
        if tok.kind == TokenKind.ENUM:
            return self.parse_enum_decl()
        if tok.kind == TokenKind.TYPE:
            return self.parse_type_alias()
        if tok.kind == TokenKind.FN:
            return self.parse_fn_decl(False)
        if tok.kind == TokenKind.IMPL:
            return self.parse_impl_block()
        if tok.kind == TokenKind.IMPORT:
            return self.parse_import_decl()
        if tok.kind == TokenKind.LET:
            return self.parse_let_decl(False)
        if tok.kind == TokenKind.DOMAIN:
            return self.parse_domain_block()
        if tok.kind == TokenKind.CONST:
            return self.parse_const_decl(False)

        raise ParseError(
            f"unexpected token {tok.kind.name} ({tok.value!r}), expected declaration",
            tok.span,
            self.filename,
        )

    def parse_decl_after_pub(self, start_span: Span) -> Decl:
        tok = self.peek()
        if tok.kind == TokenKind.STRUCT:
            return self.parse_struct_decl(True)
        if tok.kind == TokenKind.ENUM:
            return self.parse_enum_decl(True)
        if tok.kind == TokenKind.TYPE:
            return self.parse_type_alias(True)
        if tok.kind == TokenKind.FN:
            return self.parse_fn_decl(True)
        if tok.kind == TokenKind.LET:
            return self.parse_let_decl(True)
        if tok.kind == TokenKind.CONST:
            return self.parse_const_decl(True)
        raise ParseError(
            f"expected declaration after pub, got {tok.kind.name}",
            tok.span,
            self.filename,
        )

    def parse_struct_decl(self, pub: bool = False) -> StructDecl:
        start = self.expect(TokenKind.STRUCT)
        name = self.expect(TokenKind.IDENT)
        generics = self.parse_generics_opt()
        self.expect(TokenKind.LBRACE)
        fields = []
        while not self.at(TokenKind.RBRACE):
            field_pub = self.skip(TokenKind.PUB)
            field_name = self.expect(TokenKind.IDENT)
            self.expect(TokenKind.COLON)
            field_type = self.parse_type()
            fields.append(StructField(field_name.value, field_type, field_pub, field_name.span))
            self.skip(TokenKind.COMMA)
        end = self.expect(TokenKind.RBRACE)
        return StructDecl(name.value, fields, generics, Span(start.span.line, start.span.col))

    def parse_enum_decl(self, pub: bool = False) -> EnumDecl:
        start = self.expect(TokenKind.ENUM)
        name = self.expect(TokenKind.IDENT)
        generics = self.parse_generics_opt()
        self.expect(TokenKind.LBRACE)
        variants = []
        while not self.at(TokenKind.RBRACE):
            var_name = self.expect(TokenKind.IDENT)
            fields = []
            if self.skip(TokenKind.LPAREN):
                while not self.at(TokenKind.RPAREN):
                    fields.append(self.parse_type())
                    self.skip(TokenKind.COMMA)
                self.expect(TokenKind.RPAREN)
            variants.append(EnumVariant(var_name.value, fields, var_name.span))
            self.skip(TokenKind.COMMA)
        end = self.expect(TokenKind.RBRACE)
        return EnumDecl(name.value, variants, generics, Span(start.span.line, start.span.col))

    def parse_type_alias(self, pub: bool = False) -> TypeAlias:
        start = self.expect(TokenKind.TYPE)
        name = self.expect(TokenKind.IDENT)
        self.expect(TokenKind.ASSIGN)
        target = self.parse_type()
        return TypeAlias(name.value, target, Span(start.span.line, start.span.col))

    def parse_fn_decl(self, pub: bool = False) -> FnDecl:
        start = self.expect(TokenKind.FN)
        name = self.expect(TokenKind.IDENT)
        self.expect(TokenKind.LPAREN)
        params = []
        while not self.at(TokenKind.RPAREN):
            mut = self.skip(TokenKind.MUT)
            param_name = self.expect(TokenKind.IDENT)
            self.expect(TokenKind.COLON)
            param_type = self.parse_type()
            params.append(Param(param_name.value, param_type, mut, param_name.span))
            self.skip(TokenKind.COMMA)
        self.expect(TokenKind.RPAREN)

        return_type = None
        failure_type = None
        if self.skip(TokenKind.ARROW):
            return_type = self.parse_type()
            if self.skip(TokenKind.BANG):
                failure_type = self.parse_type()

        body = self.parse_block()
        return FnDecl(name.value, params, return_type, failure_type, body, pub, Span(start.span.line, start.span.col))

    def parse_impl_block(self) -> ImplBlock:
        start = self.expect(TokenKind.IMPL)
        self_type = self.expect(TokenKind.IDENT)
        self.expect(TokenKind.LBRACE)
        methods = []
        while not self.at(TokenKind.RBRACE):
            methods.append(self.parse_fn_decl())
        self.expect(TokenKind.RBRACE)
        return ImplBlock(self_type.value, methods, Span(start.span.line, start.span.col))

    def parse_import_decl(self) -> ImportDecl:
        start = self.expect(TokenKind.IMPORT)
        path = [self.expect(TokenKind.IDENT).value]
        while self.skip(TokenKind.DOUBLE_COLON):
            path.append(self.expect(TokenKind.IDENT).value)
        return ImportDecl(path, Span(start.span.line, start.span.col))

    def parse_let_decl(self, pub: bool = False) -> LetDecl:
        start = self.expect(TokenKind.LET)
        mutable = self.skip(TokenKind.MUT)
        name = self.expect(TokenKind.IDENT)
        type_ann = None
        if self.skip(TokenKind.COLON):
            type_ann = self.parse_type()
        self.expect(TokenKind.ASSIGN)
        value = self.parse_expr()
        return LetDecl(name.value, type_ann, value, mutable, pub, Span(start.span.line, start.span.col))

    def parse_const_decl(self, pub: bool = False) -> ConstDecl:
        start = self.expect(TokenKind.CONST)
        name = self.expect(TokenKind.IDENT)
        self.expect(TokenKind.COLON)
        type_ann = self.parse_type()
        self.expect(TokenKind.ASSIGN)
        value = self.parse_expr()
        return ConstDecl(name.value, type_ann, value, pub, Span(start.span.line, start.span.col))

    def parse_domain_block(self) -> DomainBlock:
        start = self.expect(TokenKind.DOMAIN)
        name = self.expect(TokenKind.IDENT)
        self.expect(TokenKind.LBRACE)
        decls = []
        while not self.at(TokenKind.RBRACE):
            decls.append(self.parse_decl())
        self.expect(TokenKind.RBRACE)
        return DomainBlock(name.value, decls, Span(start.span.line, start.span.col))

    def parse_generics_opt(self) -> List[str]:
        if not self.skip(TokenKind.LT):
            return []
        generics = [self.expect(TokenKind.IDENT).value]
        while self.skip(TokenKind.COMMA):
            generics.append(self.expect(TokenKind.IDENT).value)
        self.expect(TokenKind.GT)
        return generics

    def parse_block(self) -> Block:
        start = self.expect(TokenKind.LBRACE)
        stmts = []
        result = None

        while not self.at(TokenKind.RBRACE):
            if self.at(TokenKind.LET):
                stmts.append(self.parse_let_stmt())
            elif self.at(TokenKind.RETURN):
                stmts.append(self.parse_return_stmt())
            elif self.at(TokenKind.BREAK):
                stmts.append(self.parse_break_stmt())
            elif self.at(TokenKind.CONTINUE):
                stmts.append(self.parse_continue_stmt())
            elif self.at_expr_start():
                expr = self.parse_expr()
                if self.at(TokenKind.SEMICOLON):
                    self.advance()
                    stmts.append(ExprStmt(expr, expr.span))
                elif self.at(TokenKind.RBRACE):
                    result = expr
                else:
                    stmts.append(ExprStmt(expr, expr.span))
            else:
                break

        end = self.expect(TokenKind.RBRACE)
        return Block(stmts, result, Span(start.span.line, start.span.col))

    def parse_let_stmt(self) -> LetStmt:
        start = self.expect(TokenKind.LET)
        mutable = self.skip(TokenKind.MUT)
        name = self.expect(TokenKind.IDENT)
        type_ann = None
        if self.skip(TokenKind.COLON):
            type_ann = self.parse_type()
        self.expect(TokenKind.ASSIGN)
        value = self.parse_expr()
        self.skip(TokenKind.SEMICOLON)
        return LetStmt(name.value, type_ann, value, mutable, Span(start.span.line, start.span.col))

    def parse_return_stmt(self) -> ReturnStmt:
        start = self.expect(TokenKind.RETURN)
        value = None
        if self.at_expr_start():
            value = self.parse_expr()
        self.skip(TokenKind.SEMICOLON)
        return ReturnStmt(value, Span(start.span.line, start.span.col))

    def parse_break_stmt(self) -> BreakStmt:
        start = self.expect(TokenKind.BREAK)
        value = None
        if self.at_expr_start():
            value = self.parse_expr()
        self.skip(TokenKind.SEMICOLON)
        return BreakStmt(value, Span(start.span.line, start.span.col))

    def parse_continue_stmt(self) -> ContinueStmt:
        start = self.expect(TokenKind.CONTINUE)
        self.skip(TokenKind.SEMICOLON)
        return ContinueStmt(Span(start.span.line, start.span.col))

    # --- Expressions ---

    def parse_expr(self) -> Expr:
        return self.parse_expr_assign()

    def parse_expr_assign(self) -> Expr:
        left = self.parse_expr_or()
        if self.peek().kind in (TokenKind.ASSIGN, TokenKind.PLUS_ASSIGN, TokenKind.MINUS_ASSIGN,
                                 TokenKind.STAR_ASSIGN, TokenKind.SLASH_ASSIGN, TokenKind.PERCENT_ASSIGN):
            op = self.advance()
            right = self.parse_expr_assign()
            return BinaryOp(left, op.value, right, Span(left.span.line, left.span.col))
        return left

    def parse_expr_or(self) -> Expr:
        left = self.parse_expr_and()
        while self.at(TokenKind.OR):
            op = self.advance()
            right = self.parse_expr_and()
            left = BinaryOp(left, op.value, right, Span(left.span.line, left.span.col))
        return left

    def parse_expr_and(self) -> Expr:
        left = self.parse_expr_comparison()
        while self.at(TokenKind.AND):
            op = self.advance()
            right = self.parse_expr_comparison()
            left = BinaryOp(left, op.value, right, Span(left.span.line, left.span.col))
        return left

    def parse_expr_comparison(self) -> Expr:
        left = self.parse_expr_addition()
        while self.peek().kind in (TokenKind.EQ, TokenKind.NEQ, TokenKind.LT, TokenKind.GT,
                                    TokenKind.LTE, TokenKind.GTE):
            op = self.advance()
            right = self.parse_expr_addition()
            left = BinaryOp(left, op.value, right, Span(left.span.line, left.span.col))
        return left

    def parse_expr_addition(self) -> Expr:
        left = self.parse_expr_multiplication()
        while self.peek().kind in (TokenKind.PLUS, TokenKind.MINUS):
            op = self.advance()
            right = self.parse_expr_multiplication()
            left = BinaryOp(left, op.value, right, Span(left.span.line, left.span.col))
        return left

    def parse_expr_multiplication(self) -> Expr:
        left = self.parse_expr_unary()
        while self.peek().kind in (TokenKind.STAR, TokenKind.SLASH, TokenKind.PERCENT):
            op = self.advance()
            right = self.parse_expr_unary()
            left = BinaryOp(left, op.value, right, Span(left.span.line, left.span.col))
        return left

    def parse_expr_unary(self) -> Expr:
        tok = self.peek()

        if tok.kind == TokenKind.MINUS:
            self.advance()
            operand = self.parse_expr_unary()
            return UnaryOp("-", operand, Span(tok.span.line, tok.span.col))

        if tok.kind == TokenKind.BANG:
            self.advance()
            operand = self.parse_expr_unary()
            return UnaryOp("!", operand, Span(tok.span.line, tok.span.col))

        if tok.kind == TokenKind.AMP:
            self.advance()
            mutable = self.skip(TokenKind.MUT)
            operand = self.parse_expr_unary()
            return AddrOf(operand, mutable, Span(tok.span.line, tok.span.col))

        return self.parse_expr_postfix()

    def parse_expr_postfix(self) -> Expr:
        expr = self.parse_expr_primary()

        while True:
            if self.peek().kind == TokenKind.DOT:
                self.advance()
                field = self.expect(TokenKind.IDENT)
                expr = FieldAccess(expr, field.value, Span(expr.span.line, expr.span.col))
            elif self.peek().kind == TokenKind.LBRACKET:
                self.advance()
                index = self.parse_expr()
                self.expect(TokenKind.RBRACKET)
                expr = IndexAccess(expr, index, Span(expr.span.line, expr.span.col))
            elif self.peek().kind == TokenKind.LPAREN:
                self.advance()
                args = []
                while not self.at(TokenKind.RPAREN):
                    args.append(self.parse_expr())
                    self.skip(TokenKind.COMMA)
                self.expect(TokenKind.RPAREN)
                expr = Call(expr, args, Span(expr.span.line, expr.span.col))
            elif self.peek().kind == TokenKind.AWAIT:
                self.advance()
                expr = Call(
                    Ident("await", expr.span),
                    [expr],
                    Span(expr.span.line, expr.span.col),
                )
            elif self.peek().kind == TokenKind.QUESTION:
                self.advance()
                expr = Propagation(expr, Span(expr.span.line, expr.span.col))
            else:
                break

        return expr

    def parse_expr_primary(self) -> Expr:
        tok = self.peek()

        if tok.kind == TokenKind.INT:
            self.advance()
            return IntLit(tok.value, tok.span)

        if tok.kind == TokenKind.FLOAT:
            self.advance()
            return FloatLit(tok.value, tok.span)

        if tok.kind == TokenKind.STRING:
            self.advance()
            return StringLit(tok.value, tok.span)

        if tok.kind == TokenKind.CHAR:
            self.advance()
            return CharLit(tok.value, tok.span)

        if tok.kind in (TokenKind.TRUE, TokenKind.FALSE):
            self.advance()
            return BoolLit(tok.value == "true", tok.span)

        if tok.kind == TokenKind.IDENT:
            name = self.advance()
            if self.peek().kind == TokenKind.LBRACE and name.value[0].isupper():
                return self.parse_struct_construct(name)
            return Ident(name.value, name.span)

        if tok.kind == TokenKind.LPAREN:
            return self.parse_tuple_or_paren()

        if tok.kind == TokenKind.LBRACE:
            return BlockExpr(self.parse_block(), tok.span)

        if tok.kind == TokenKind.IF:
            return self.parse_if_expr()

        if tok.kind == TokenKind.WHILE:
            return self.parse_while_expr()

        if tok.kind == TokenKind.FOR:
            return self.parse_for_expr()

        if tok.kind == TokenKind.MATCH:
            return self.parse_match_expr()

        if tok.kind == TokenKind.RETURN:
            return self.parse_return_expr()

        if tok.kind == TokenKind.BREAK:
            self.advance()
            return BreakExpr(None, tok.span)

        if tok.kind == TokenKind.CONTINUE:
            self.advance()
            return ContinueExpr(tok.span)

        if tok.kind == TokenKind.SOME:
            self.advance()
            self.expect(TokenKind.LPAREN)
            value = self.parse_expr()
            self.expect(TokenKind.RPAREN)
            return MaybeSome(value, tok.span)

        if tok.kind == TokenKind.NONE:
            self.advance()
            return MaybeNone(tok.span)

        if tok.kind == TokenKind.PANIC:
            self.advance()
            self.expect(TokenKind.LPAREN)
            msg = self.parse_expr()
            self.expect(TokenKind.RPAREN)
            return Call(Ident("panic", tok.span), [msg], tok.span)

        if tok.kind == TokenKind.UNREACHABLE:
            self.advance()
            return Ident("unreachable", tok.span)

        raise ParseError(
            f"unexpected token {tok.kind.name} ({tok.value!r}), expected expression",
            tok.span,
            self.filename,
        )

    def parse_struct_construct(self, name: Token) -> StructConstruct:
        self.expect(TokenKind.LBRACE)
        fields = []
        base = None
        while not self.at(TokenKind.RBRACE):
            if self.skip(TokenKind.DOUBLE_DOT):
                base = self.parse_expr()
                break
            field_name = self.expect(TokenKind.IDENT)
            self.expect(TokenKind.COLON)
            field_value = self.parse_expr()
            fields.append((field_name.value, field_value))
            self.skip(TokenKind.COMMA)
        self.expect(TokenKind.RBRACE)
        return StructConstruct(name.value, fields, base, Span(name.span.line, name.span.col))

    def parse_tuple_or_paren(self) -> Expr:
        start = self.expect(TokenKind.LPAREN)
        if self.at(TokenKind.RPAREN):
            self.advance()
            return UnitLit(Span(start.span.line, start.span.col))
        first = self.parse_expr()
        if self.at(TokenKind.COMMA):
            elements = [first]
            while self.skip(TokenKind.COMMA):
                if self.at(TokenKind.RPAREN):
                    break
                elements.append(self.parse_expr())
            self.expect(TokenKind.RPAREN)
            return TupleConstruct(elements, Span(start.span.line, start.span.col))
        self.expect(TokenKind.RPAREN)
        return first

    def parse_if_expr(self) -> IfExpr:
        start = self.expect(TokenKind.IF)
        condition = self.parse_expr()
        then_block = self.parse_block()
        else_block = None
        if self.skip(TokenKind.ELSE):
            if self.at(TokenKind.IF):
                else_block = self.parse_if_expr()
            else:
                else_block = self.parse_block()
        return IfExpr(condition, then_block, else_block, Span(start.span.line, start.span.col))

    def parse_while_expr(self) -> WhileExpr:
        start = self.expect(TokenKind.WHILE)
        condition = self.parse_expr()
        body = self.parse_block()
        return WhileExpr(condition, body, Span(start.span.line, start.span.col))

    def parse_for_expr(self) -> ForExpr:
        start = self.expect(TokenKind.FOR)
        var = self.expect(TokenKind.IDENT)
        self.expect(TokenKind.IN)
        iterable = self.parse_expr()
        body = self.parse_block()
        return ForExpr(var.value, iterable, body, Span(start.span.line, start.span.col))

    def parse_match_expr(self) -> MatchExpr:
        start = self.expect(TokenKind.MATCH)
        subject = self.parse_expr()
        self.expect(TokenKind.LBRACE)
        arms = []
        while not self.at(TokenKind.RBRACE):
            pattern = self.parse_pattern()
            self.expect(TokenKind.FAT_ARROW)
            body = self.parse_expr()
            self.skip(TokenKind.COMMA)
            arms.append(MatchArm(pattern, body, Span(pattern.span.line, pattern.span.col)))
        self.expect(TokenKind.RBRACE)
        return MatchExpr(subject, arms, Span(start.span.line, start.span.col))

    def parse_return_expr(self) -> ReturnExpr:
        start = self.expect(TokenKind.RETURN)
        value = None
        if self.at_expr_start():
            value = self.parse_expr()
        return ReturnExpr(value, Span(start.span.line, start.span.col))

    # --- Patterns ---

    def parse_pattern(self) -> Pattern:
        return self.parse_pattern_or()

    def parse_pattern_or(self) -> Pattern:
        first = self.parse_pattern_primary()
        if self.at(TokenKind.PIPE):
            patterns = [first]
            while self.skip(TokenKind.PIPE):
                patterns.append(self.parse_pattern_primary())
            return OrPattern(patterns, Span(first.span.line, first.span.col))
        return first

    def parse_pattern_primary(self) -> Pattern:
        tok = self.peek()

        if tok.kind == TokenKind.UNDERSCORE:
            self.advance()
            return WildcardPattern(tok.span)

        if tok.kind in (TokenKind.INT, TokenKind.STRING, TokenKind.CHAR):
            self.advance()
            return LiteralPattern(tok.value, tok.span)

        if tok.kind in (TokenKind.TRUE, TokenKind.FALSE):
            self.advance()
            return LiteralPattern(tok.value, tok.span)

        if tok.kind == TokenKind.IDENT:
            name = self.advance()
            if self.peek().kind == TokenKind.LPAREN:
                self.advance()
                inner = []
                if not self.at(TokenKind.RPAREN):
                    inner.append(self.parse_pattern())
                    while self.skip(TokenKind.COMMA):
                        if self.at(TokenKind.RPAREN):
                            break
                        inner.append(self.parse_pattern())
                self.expect(TokenKind.RPAREN)
                return EnumPattern(None, name.value, inner, tok.span)
            return IdentPattern(name.value, tok.span)

        if tok.kind == TokenKind.LPAREN:
            self.advance()
            elements = []
            if not self.at(TokenKind.RPAREN):
                elements.append(self.parse_pattern())
                while self.skip(TokenKind.COMMA):
                    if self.at(TokenKind.RPAREN):
                        break
                    elements.append(self.parse_pattern())
            self.expect(TokenKind.RPAREN)
            return TuplePattern(elements, tok.span)

        raise ParseError(
            f"unexpected token {tok.kind.name} ({tok.value!r}), expected pattern",
            tok.span,
            self.filename,
        )

    # --- Types ---

    def parse_type(self) -> Type:
        return self.parse_type_ref()

    def parse_type_ref(self) -> Type:
        if self.at(TokenKind.AMP):
            start = self.advance()
            mutable = self.skip(TokenKind.MUT)
            inner = self.parse_type_ref()
            return RefType(inner, mutable, Span(start.span.line, start.span.col))

        if self.at(TokenKind.MAYBE):
            start = self.advance()
            inner = self.parse_type_ref()
            return MaybeType(inner, Span(start.span.line, start.span.col))

        return self.parse_type_base()

    def parse_type_base(self) -> Type:
        tok = self.peek()

        if tok.kind == TokenKind.LPAREN:
            return self.parse_type_tuple()

        if tok.kind == TokenKind.LBRACKET:
            return self.parse_type_array()

        if tok.kind == TokenKind.FN:
            return self.parse_type_fn()

        self.expect(TokenKind.IDENT)
        name = tok.value

        PRIMITIVES = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char", "String"}

        if name in PRIMITIVES:
            return PrimitiveType(name, tok.span)

        args = []
        if self.skip(TokenKind.LT):
            args.append(self.parse_type())
            while self.skip(TokenKind.COMMA):
                args.append(self.parse_type())
            self.expect(TokenKind.GT)

        return NamedType(name, args, tok.span)

    def parse_type_fn(self) -> Type:
        start = self.expect(TokenKind.FN)
        self.expect(TokenKind.LPAREN)
        params = []
        if not self.at(TokenKind.RPAREN):
            params.append(self.parse_type())
            while self.skip(TokenKind.COMMA):
                params.append(self.parse_type())
        self.expect(TokenKind.RPAREN)
        self.expect(TokenKind.ARROW)
        return_type = self.parse_type()
        return FnType(params, return_type, Span(start.span.line, start.span.col))

    def parse_type_tuple(self) -> Type:
        start = self.expect(TokenKind.LPAREN)
        if self.at(TokenKind.RPAREN):
            self.advance()
            return TupleType([], Span(start.span.line, start.span.col))
        elements = [self.parse_type()]
        while self.skip(TokenKind.COMMA):
            if self.at(TokenKind.RPAREN):
                break
            elements.append(self.parse_type())
        self.expect(TokenKind.RPAREN)
        if len(elements) == 1:
            return elements[0]
        return TupleType(elements, Span(start.span.line, start.span.col))

    def parse_type_array(self) -> Type:
        start = self.expect(TokenKind.LBRACKET)
        inner = self.parse_type()
        self.expect(TokenKind.SEMICOLON)
        size_tok = self.expect(TokenKind.INT)
        self.expect(TokenKind.RBRACKET)
        return ArrayType(inner, int(size_tok.value), Span(start.span.line, start.span.col))
