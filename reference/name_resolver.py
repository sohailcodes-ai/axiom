from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set
from ast_nodes import *
from errors import ResolveError
from tokens import Span


@dataclass
class NameBinding:
    kind: str  # "fn", "struct", "enum", "let", "param", "module"
    name: str
    span: Span
    value: object = None


class NameResolver:
    def __init__(self, filename: str = "<input>"):
        self.filename = filename
        self.scopes: List[Dict[str, NameBinding]] = [{}]
        self.modules: Dict[str, Dict[str, NameBinding]] = {}
        self.errors: List[ResolveError] = []
        self.current_function: Optional[str] = None
        self.loop_depth: int = 0

        # Built-in functions
        builtins = ["print"]
        for name in builtins:
            self.scopes[-1][name] = NameBinding("fn", name, Span(0, 0))

    def push_scope(self) -> None:
        self.scopes.append({})

    def pop_scope(self) -> None:
        self.scopes.pop()

    def define(self, binding: NameBinding) -> None:
        self.scopes[-1][binding.name] = binding

    def resolve(self, name: str, span: Span) -> Optional[NameBinding]:
        for scope in reversed(self.scopes):
            if name in scope:
                return scope[name]
        self.errors.append(ResolveError(f"undefined name: {name}", span, self.filename))
        return None

    def resolve_module(self, parts: List[str], span: Span) -> Optional[NameBinding]:
        if not parts:
            return None

        first = parts[0]
        binding = self.resolve(first, span)
        if binding is None:
            return None

        if binding.kind == "module" and len(parts) > 1:
            mod = self.modules.get(first, {})
            for part in parts[1:]:
                if part in mod:
                    binding = mod[part]
                else:
                    self.errors.append(
                        ResolveError(f"module {first} has no member {part}", span, self.filename)
                    )
                    return None
            return binding

        return binding

    def resolve_program(self, program: Program) -> None:
        for decl in program.declarations:
            self.resolve_decl(decl)

    def resolve_decl(self, decl: Decl) -> None:
        if isinstance(decl, FnDecl):
            self.define(NameBinding("fn", decl.name, decl.span))
            self.push_scope()
            for param in decl.params:
                self.define(NameBinding("param", param.name, param.span))
            self.current_function = decl.name
            self.resolve_block(decl.body)
            self.current_function = None
            self.pop_scope()

        elif isinstance(decl, StructDecl):
            self.define(NameBinding("struct", decl.name, decl.span))

        elif isinstance(decl, EnumDecl):
            self.define(NameBinding("enum", decl.name, decl.span))

        elif isinstance(decl, TypeAlias):
            self.define(NameBinding("type", decl.name, decl.span))

        elif isinstance(decl, LetDecl):
            self.resolve_expr(decl.value)
            self.define(NameBinding("let", decl.name, decl.span))

        elif isinstance(decl, ConstDecl):
            self.resolve_expr(decl.value)
            self.define(NameBinding("const", decl.name, decl.span))

        elif isinstance(decl, ImportDecl):
            mod_name = decl.path[0]
            self.define(NameBinding("module", mod_name, decl.span))
            self.modules[mod_name] = {}

        elif isinstance(decl, ImplBlock):
            for method in decl.methods:
                self.resolve_decl(method)

        elif isinstance(decl, DomainBlock):
            for inner_decl in decl.declarations:
                self.resolve_decl(inner_decl)

    def resolve_block(self, block: Block) -> None:
        self.push_scope()
        for stmt in block.stmts:
            self.resolve_stmt(stmt)
        if block.result is not None:
            self.resolve_expr(block.result)
        self.pop_scope()

    def resolve_stmt(self, stmt: Stmt) -> None:
        if isinstance(stmt, LetStmt):
            self.resolve_expr(stmt.value)
            self.define(NameBinding("let", stmt.name, stmt.span))

        elif isinstance(stmt, ExprStmt):
            self.resolve_expr(stmt.expr)

        elif isinstance(stmt, ReturnStmt):
            if stmt.value:
                self.resolve_expr(stmt.value)

        elif isinstance(stmt, BreakStmt):
            if self.loop_depth == 0:
                self.errors.append(ResolveError("break outside of loop", stmt.span, self.filename))

        elif isinstance(stmt, ContinueStmt):
            if self.loop_depth == 0:
                self.errors.append(ResolveError("continue outside of loop", stmt.span, self.filename))

    def resolve_expr(self, expr: Expr) -> None:
        if isinstance(expr, IntLit):
            pass
        elif isinstance(expr, FloatLit):
            pass
        elif isinstance(expr, StringLit):
            pass
        elif isinstance(expr, CharLit):
            pass
        elif isinstance(expr, BoolLit):
            pass
        elif isinstance(expr, UnitLit):
            pass

        elif isinstance(expr, Ident):
            if expr.name == "unreachable":
                return
            self.resolve(expr.name, expr.span)

        elif isinstance(expr, FieldAccess):
            self.resolve_expr(expr.object)

        elif isinstance(expr, IndexAccess):
            self.resolve_expr(expr.object)
            self.resolve_expr(expr.index)

        elif isinstance(expr, Call):
            self.resolve_expr(expr.callee)
            for arg in expr.args:
                self.resolve_expr(arg)

        elif isinstance(expr, UnaryOp):
            self.resolve_expr(expr.operand)

        elif isinstance(expr, BinaryOp):
            self.resolve_expr(expr.left)
            self.resolve_expr(expr.right)

        elif isinstance(expr, IfExpr):
            self.resolve_expr(expr.condition)
            self.resolve_block(expr.then_block)
            if expr.else_block:
                if isinstance(expr.else_block, Block):
                    self.resolve_block(expr.else_block)
                else:
                    self.resolve_expr(expr.else_block)

        elif isinstance(expr, WhileExpr):
            self.resolve_expr(expr.condition)
            self.loop_depth += 1
            self.resolve_block(expr.body)
            self.loop_depth -= 1

        elif isinstance(expr, ForExpr):
            self.resolve_expr(expr.iterable)
            self.push_scope()
            self.define(NameBinding("let", expr.var, expr.span))
            self.loop_depth += 1
            self.resolve_block(expr.body)
            self.loop_depth -= 1
            self.pop_scope()

        elif isinstance(expr, MatchExpr):
            self.resolve_expr(expr.subject)
            for arm in expr.arms:
                self.push_scope()
                self.resolve_pattern(arm.pattern)
                self.resolve_expr(arm.body)
                self.pop_scope()

        elif isinstance(expr, ReturnExpr):
            if expr.value:
                self.resolve_expr(expr.value)

        elif isinstance(expr, BreakExpr):
            if expr.value:
                self.resolve_expr(expr.value)

        elif isinstance(expr, ContinueExpr):
            pass

        elif isinstance(expr, BlockExpr):
            self.resolve_block(expr.block)

        elif isinstance(expr, StructConstruct):
            self.resolve(expr.name, expr.span)
            for _, value in expr.fields:
                self.resolve_expr(value)
            if expr.base:
                self.resolve_expr(expr.base)

        elif isinstance(expr, TupleConstruct):
            for elem in expr.elements:
                self.resolve_expr(elem)

        elif isinstance(expr, MaybeSome):
            self.resolve_expr(expr.value)

        elif isinstance(expr, MaybeNone):
            pass

        elif isinstance(expr, Propagation):
            self.resolve_expr(expr.expr)

        elif isinstance(expr, AddrOf):
            self.resolve_expr(expr.expr)

    def resolve_pattern(self, pattern: Pattern) -> None:
        if isinstance(pattern, LiteralPattern):
            pass
        elif isinstance(pattern, WildcardPattern):
            pass
        elif isinstance(pattern, IdentPattern):
            self.define(NameBinding("let", pattern.name, pattern.span))
        elif isinstance(pattern, TuplePattern):
            for elem in pattern.elements:
                self.resolve_pattern(elem)
        elif isinstance(pattern, EnumPattern):
            if pattern.module:
                self.resolve_module([pattern.module, pattern.name], pattern.span)
            else:
                self.resolve(pattern.name, pattern.span)
            if pattern.inner:
                for p in pattern.inner:
                    self.resolve_pattern(p)
        elif isinstance(pattern, OrPattern):
            for p in pattern.patterns:
                self.resolve_pattern(p)
