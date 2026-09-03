from dataclasses import dataclass, field
from typing import List, Optional
from tokens import Span


# --- Expressions ---

@dataclass
class IntLit:
    value: str
    span: Span


@dataclass
class FloatLit:
    value: str
    span: Span


@dataclass
class StringLit:
    value: str
    span: Span


@dataclass
class CharLit:
    value: str
    span: Span


@dataclass
class BoolLit:
    value: bool
    span: Span


@dataclass
class UnitLit:
    span: Span


@dataclass
class Ident:
    name: str
    span: Span


@dataclass
class FieldAccess:
    object: "Expr"
    field: str
    span: Span


@dataclass
class IndexAccess:
    object: "Expr"
    index: "Expr"
    span: Span


@dataclass
class Call:
    callee: "Expr"
    args: List["Expr"]
    span: Span


@dataclass
class UnaryOp:
    op: str
    operand: "Expr"
    span: Span


@dataclass
class BinaryOp:
    left: "Expr"
    op: str
    right: "Expr"
    span: Span


@dataclass
class IfExpr:
    condition: "Expr"
    then_block: "Block"
    else_block: Optional["Block | IfExpr"]
    span: Span


@dataclass
class WhileExpr:
    condition: "Expr"
    body: "Block"
    span: Span


@dataclass
class ForExpr:
    var: str
    iterable: "Expr"
    body: "Block"
    span: Span


@dataclass
class MatchExpr:
    subject: "Expr"
    arms: List["MatchArm"]
    span: Span


@dataclass
class MatchArm:
    pattern: "Pattern"
    body: "Expr"
    span: Span


@dataclass
class ReturnExpr:
    value: Optional["Expr"]
    span: Span


@dataclass
class BreakExpr:
    value: Optional["Expr"]
    span: Span


@dataclass
class ContinueExpr:
    span: Span


@dataclass
class BlockExpr:
    block: "Block"
    span: Span


@dataclass
class StructConstruct:
    name: str
    fields: List[tuple[str, "Expr"]]
    base: Optional["Expr"]
    span: Span


@dataclass
class MaybeSome:
    value: "Expr"
    span: Span


@dataclass
class MaybeNone:
    span: Span


@dataclass
class Propagation:
    expr: "Expr"
    span: Span


@dataclass
class AddrOf:
    expr: "Expr"
    mutable: bool
    span: Span


@dataclass
class TupleConstruct:
    elements: List["Expr"]
    span: Span


Expr = (
    IntLit | FloatLit | StringLit | CharLit | BoolLit | UnitLit
    | Ident | FieldAccess | IndexAccess | Call
    | UnaryOp | BinaryOp
    | IfExpr | WhileExpr | ForExpr | MatchExpr | ReturnExpr | BreakExpr | ContinueExpr
    | BlockExpr | StructConstruct | TupleConstruct
    | MaybeSome | MaybeNone | Propagation | AddrOf
)


# --- Patterns ---

@dataclass
class LiteralPattern:
    value: str
    span: Span


@dataclass
class IdentPattern:
    name: str
    span: Span


@dataclass
class WildcardPattern:
    span: Span


@dataclass
class TuplePattern:
    elements: List["Pattern"]
    span: Span


@dataclass
class EnumPattern:
    module: Optional[str]
    name: str
    inner: Optional[List["Pattern"]]
    span: Span


@dataclass
class OrPattern:
    patterns: List["Pattern"]
    span: Span


Pattern = LiteralPattern | IdentPattern | WildcardPattern | TuplePattern | EnumPattern | OrPattern


# --- Types ---

@dataclass
class PrimitiveType:
    name: str
    span: Span


@dataclass
class NamedType:
    name: str
    args: List["Type"]
    span: Span


@dataclass
class RefType:
    inner: "Type"
    mutable: bool
    span: Span


@dataclass
class MaybeType:
    inner: "Type"
    span: Span


@dataclass
class FailureType:
    ok: "Type"
    err: "Type"
    span: Span


@dataclass
class FnType:
    params: List["Type"]
    return_type: "Type"
    span: Span


@dataclass
class TupleType:
    elements: List["Type"]
    span: Span


@dataclass
class ArrayType:
    inner: "Type"
    size: int
    span: Span


Type = PrimitiveType | NamedType | RefType | MaybeType | FailureType | FnType | TupleType | ArrayType


# --- Declarations ---

@dataclass
class StructField:
    name: str
    type: Type
    pub: bool
    span: Span


@dataclass
class StructDecl:
    name: str
    fields: List[StructField]
    generics: List[str]
    span: Span


@dataclass
class EnumVariant:
    name: str
    fields: List[Type]
    span: Span


@dataclass
class EnumDecl:
    name: str
    variants: List[EnumVariant]
    generics: List[str]
    span: Span


@dataclass
class TypeAlias:
    name: str
    target: Type
    span: Span


@dataclass
class Param:
    name: str
    type: Type
    mutable: bool
    span: Span


@dataclass
class FnDecl:
    name: str
    params: List[Param]
    return_type: Optional[Type]
    failure_type: Optional[Type]
    body: "Block"
    pub: bool
    span: Span


@dataclass
class ImplBlock:
    self_type: str
    methods: List[FnDecl]
    span: Span


@dataclass
class ImportDecl:
    path: List[str]
    span: Span


@dataclass
class LetDecl:
    name: str
    type: Optional[Type]
    value: Expr
    mutable: bool
    pub: bool
    span: Span


@dataclass
class DomainBlock:
    name: str
    declarations: List["Decl"]
    span: Span


@dataclass
class ConstDecl:
    name: str
    type: Type
    value: Expr
    pub: bool
    span: Span


Decl = StructDecl | EnumDecl | TypeAlias | FnDecl | ImplBlock | ImportDecl | LetDecl | DomainBlock | ConstDecl


# --- Block ---

@dataclass
class Block:
    stmts: List["Stmt"]
    result: Optional[Expr]
    span: Span


# --- Statements ---

@dataclass
class LetStmt:
    name: str
    type: Optional[Type]
    value: Expr
    mutable: bool
    span: Span


@dataclass
class ExprStmt:
    expr: Expr
    span: Span


@dataclass
class ReturnStmt:
    value: Optional[Expr]
    span: Span


@dataclass
class BreakStmt:
    value: Optional[Expr]
    span: Span


@dataclass
class ContinueStmt:
    span: Span


Stmt = LetStmt | ExprStmt | ReturnStmt | BreakStmt | ContinueStmt


# --- Program ---

@dataclass
class Program:
    declarations: List[Decl]
    span: Span
