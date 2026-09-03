from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any
from tokens import Span


@dataclass
class HIRFunction:
    name: str
    params: List[str]
    body: "HIRBlock"
    span: Span


@dataclass
class HIRBlock:
    stmts: List["HIRStmt"]
    result: Optional["HIRExpr"]
    span: Span


@dataclass
class HIRStmt:
    kind: str  # "let", "assign", "expr"
    name: Optional[str] = None
    value: Optional["HIRExpr"] = None
    span: Optional[Span] = None


@dataclass
class HIRExpr:
    kind: str  # "int", "float", "string", "bool", "unit", "ident", "binary", "unary", "call", "if", "while", "field", "struct_construct", "maybe_some", "maybe_none"
    value: Any = None
    left: Optional["HIRExpr"] = None
    right: Optional["HIRExpr"] = None
    op: Optional[str] = None
    callee: Optional["HIRExpr"] = None
    args: Optional[List["HIRExpr"]] = None
    condition: Optional["HIRExpr"] = None
    then_block: Optional["HIRBlock"] = None
    else_block: Optional["HIRBlock"] = None
    body: Optional["HIRBlock"] = None
    object: Optional["HIRExpr"] = None
    field: Optional[str] = None
    fields: Optional[Dict[str, "HIRExpr"]] = None
    span: Optional[Span] = None


@dataclass
class HIRProgram:
    functions: List[HIRFunction]
    structs: Dict[str, Dict[str, str]]  # name -> {field: type}
    enums: Dict[str, Dict[str, List[str]]]  # name -> {variant: [types]}
    imports: List[str]
    span: Span
