from dataclasses import dataclass, field
from typing import Dict, List, Optional, Any
from ast_nodes import *
from errors import TypeError as TypeError_
from tokens import Span


@dataclass
class TypeValue:
    kind: str  # "i64", "f64", "bool", "string", "unit", "unknown", "function", "struct", "enum", "maybe", "failure"
    name: str = ""
    params: List["TypeValue"] = field(default_factory=list)
    return_type: Optional["TypeValue"] = None
    failure_type: Optional["TypeValue"] = None
    fields: Dict[str, "TypeValue"] = field(default_factory=dict)
    variants: Dict[str, List["TypeValue"]] = field(default_factory=dict)

    def __eq__(self, other):
        if not isinstance(other, TypeValue):
            return False
        if self.kind != other.kind:
            return False
        if self.kind in ("i64", "f64", "bool", "string", "unit"):
            return True
        if self.kind == "maybe":
            return self.params[0] == other.params[0] if self.params and other.params else True
        if self.kind == "failure":
            return (self.params[0] == other.params[0] if self.params and other.params else True) and \
                   (self.params[1] == other.params[1] if len(self.params) > 1 and len(other.params) > 1 else True)
        return True

    def is_compatible(self, other: "TypeValue") -> bool:
        if self.kind == "unknown" or other.kind == "unknown":
            return True
        return self == other


I64 = TypeValue("i64")
F64 = TypeValue("f64")
BOOL = TypeValue("bool")
STRING = TypeValue("string")
UNIT = TypeValue("unit")
UNKNOWN = TypeValue("unknown")


class TypeChecker:
    def __init__(self, filename: str = "<input>"):
        self.filename = filename
        self.types: Dict[str, TypeValue] = {}
        self.expr_types: Dict[int, TypeValue] = {}
        self.errors: List[TypeError_] = []
        self.current_function: Optional[str] = None

    def check_program(self, program: Program) -> None:
        for decl in program.declarations:
            self.check_decl(decl)

    def check_decl(self, decl: Decl) -> None:
        if isinstance(decl, FnDecl):
            self.check_fn_decl(decl)
        elif isinstance(decl, StructDecl):
            self.check_struct_decl(decl)
        elif isinstance(decl, EnumDecl):
            self.check_enum_decl(decl)
        elif isinstance(decl, LetDecl):
            self.check_let_decl(decl)
        elif isinstance(decl, ImplBlock):
            for method in decl.methods:
                self.check_fn_decl(method)
        elif isinstance(decl, DomainBlock):
            for inner_decl in decl.declarations:
                self.check_decl(inner_decl)

    def check_fn_decl(self, decl: FnDecl) -> None:
        self.current_function = decl.name
        param_types = []
        for param in decl.params:
            param_type = self.resolve_type(param.type)
            param_types.append(param_type)
            self.types[param.name] = param_type

        return_type = self.resolve_type(decl.return_type) if decl.return_type else UNIT
        failure_type = self.resolve_type(decl.failure_type) if decl.failure_type else None

        self.types[decl.name] = TypeValue(
            "function",
            params=param_types,
            return_type=return_type,
            failure_type=failure_type,
        )

        body_type = self.check_block(decl.body, return_type)
        if not body_type.is_compatible(return_type):
            self.errors.append(
                TypeError_(
                    f"function {decl.name} returns {return_type.kind}, but body returns {body_type.kind}",
                    decl.span,
                    self.filename,
                )
            )
        self.current_function = None

    def check_struct_decl(self, decl: StructDecl) -> None:
        fields = {}
        for field in decl.fields:
            fields[field.name] = self.resolve_type(field.type)
        self.types[decl.name] = TypeValue("struct", name=decl.name, fields=fields)

    def check_enum_decl(self, decl: EnumDecl) -> None:
        variants = {}
        for variant in decl.variants:
            variant_types = [self.resolve_type(t) for t in variant.fields]
            variants[variant.name] = variant_types
        self.types[decl.name] = TypeValue("enum", name=decl.name, variants=variants)

    def check_let_decl(self, decl: LetDecl) -> None:
        value_type = self.check_expr(decl.value)
        if decl.type:
            declared_type = self.resolve_type(decl.type)
            if not value_type.is_compatible(declared_type):
                self.errors.append(
                    TypeError_(
                        f"let {decl.name}: {declared_type.kind} = {value_type.kind}",
                        decl.span,
                        self.filename,
                    )
                )
        self.types[decl.name] = value_type

    def check_block(self, block: Block, expected_return: TypeValue = UNIT) -> TypeValue:
        for stmt in block.stmts:
            self.check_stmt(stmt)
        if block.result:
            return self.check_expr(block.result)
        return UNIT

    def check_stmt(self, stmt: Stmt) -> None:
        if isinstance(stmt, LetStmt):
            value_type = self.check_expr(stmt.value)
            if stmt.type:
                declared_type = self.resolve_type(stmt.type)
                if not value_type.is_compatible(declared_type):
                    self.errors.append(
                        TypeError_(
                            f"let {stmt.name}: {declared_type.kind} = {value_type.kind}",
                            stmt.span,
                            self.filename,
                        )
                    )
            self.types[stmt.name] = value_type

        elif isinstance(stmt, ExprStmt):
            self.check_expr(stmt.expr)

        elif isinstance(stmt, ReturnStmt):
            if stmt.value:
                self.check_expr(stmt.value)

    def check_expr(self, expr: Expr) -> TypeValue:
        if isinstance(expr, IntLit):
            return I64

        if isinstance(expr, FloatLit):
            return F64

        if isinstance(expr, StringLit):
            return STRING

        if isinstance(expr, CharLit):
            return I64

        if isinstance(expr, BoolLit):
            return BOOL

        if isinstance(expr, UnitLit):
            return UNIT

        if isinstance(expr, Ident):
            if expr.name == "unreachable":
                return UNKNOWN
            if expr.name in self.types:
                return self.types[expr.name]
            return UNKNOWN

        if isinstance(expr, FieldAccess):
            obj_type = self.check_expr(expr.object)
            if obj_type.kind == "struct" and expr.field in obj_type.fields:
                return obj_type.fields[expr.field]
            return UNKNOWN

        if isinstance(expr, Call):
            callee_type = self.check_expr(expr.callee)
            if callee_type.kind == "function":
                for i, arg in enumerate(expr.args):
                    arg_type = self.check_expr(arg)
                    if i < len(callee_type.params):
                        if not arg_type.is_compatible(callee_type.params[i]):
                            self.errors.append(
                                TypeError_(
                                    f"argument {i+1}: expected {callee_type.params[i].kind}, got {arg_type.kind}",
                                    expr.span,
                                    self.filename,
                                )
                            )
                return callee_type.return_type or UNIT

            if isinstance(expr.callee, Ident) and expr.callee.name == "print":
                for arg in expr.args:
                    self.check_expr(arg)
                return UNIT

            return UNKNOWN

        if isinstance(expr, BinaryOp):
            left_type = self.check_expr(expr.left)
            right_type = self.check_expr(expr.right)
            if expr.op in ("+", "-", "*", "/", "%"):
                if left_type.kind == "i64" and right_type.kind == "i64":
                    return I64
                if left_type.kind == "f64" and right_type.kind == "f64":
                    return F64
                if expr.op == "+" and left_type.kind == "string" and right_type.kind == "string":
                    return STRING
                return UNKNOWN
            if expr.op in ("==", "!=", "<", ">", "<=", ">="):
                return BOOL
            if expr.op in ("&&", "||"):
                return BOOL
            return UNKNOWN

        if isinstance(expr, UnaryOp):
            operand_type = self.check_expr(expr.operand)
            if expr.op == "-" and operand_type.kind == "i64":
                return I64
            if expr.op == "!" and operand_type.kind == "bool":
                return BOOL
            return UNKNOWN

        if isinstance(expr, IfExpr):
            self.check_expr(expr.condition)
            then_type = self.check_block(expr.then_block)
            if expr.else_block:
                if isinstance(expr.else_block, Block):
                    else_type = self.check_block(expr.else_block)
                else:
                    else_type = self.check_expr(expr.else_block)
                if then_type.is_compatible(else_type):
                    return then_type
                return UNKNOWN
            return then_type

        if isinstance(expr, WhileExpr):
            self.check_expr(expr.condition)
            self.check_block(expr.body)
            return UNIT

        if isinstance(expr, ForExpr):
            self.check_expr(expr.iterable)
            self.check_block(expr.body)
            return UNIT

        if isinstance(expr, MatchExpr):
            self.check_expr(expr.subject)
            for arm in expr.arms:
                self.check_expr(arm.body)
            return UNKNOWN

        if isinstance(expr, ReturnExpr):
            return UNKNOWN

        if isinstance(expr, BreakExpr):
            return UNKNOWN

        if isinstance(expr, ContinueExpr):
            return UNKNOWN

        if isinstance(expr, BlockExpr):
            return self.check_block(expr.block)

        if isinstance(expr, StructConstruct):
            if expr.name in self.types:
                struct_type = self.types[expr.name]
                if struct_type.kind == "struct":
                    return struct_type
            return UNKNOWN

        if isinstance(expr, TupleConstruct):
            elem_types = [self.check_expr(e) for e in expr.elements]
            return TypeValue("tuple", params=elem_types)

        if isinstance(expr, MaybeSome):
            inner_type = self.check_expr(expr.value)
            return TypeValue("maybe", params=[inner_type])

        if isinstance(expr, MaybeNone):
            return TypeValue("maybe", params=[UNKNOWN])

        if isinstance(expr, Propagation):
            self.check_expr(expr.expr)
            return UNKNOWN

        if isinstance(expr, AddrOf):
            inner_type = self.check_expr(expr.expr)
            if expr.mutable:
                return TypeValue("ref_mut", params=[inner_type])
            return TypeValue("ref", params=[inner_type])

        return UNKNOWN

    def resolve_type(self, type_node: Optional[Type]) -> TypeValue:
        if type_node is None:
            return UNIT

        if isinstance(type_node, PrimitiveType):
            PRIMITIVES = {
                "i8": I64, "i16": I64, "i32": I64, "i64": I64,
                "u8": I64, "u16": I64, "u32": I64, "u64": I64,
                "f32": F64, "f64": F64,
                "bool": BOOL, "char": I64, "String": STRING,
            }
            return PRIMITIVES.get(type_node.name, UNKNOWN)

        if isinstance(type_node, NamedType):
            if type_node.name in self.types:
                return self.types[type_node.name]
            return TypeValue("struct", name=type_node.name)

        if isinstance(type_node, RefType):
            inner = self.resolve_type(type_node.inner)
            if type_node.mutable:
                return TypeValue("ref_mut", params=[inner])
            return TypeValue("ref", params=[inner])

        if isinstance(type_node, MaybeType):
            inner = self.resolve_type(type_node.inner)
            return TypeValue("maybe", params=[inner])

        if isinstance(type_node, FailureType):
            ok = self.resolve_type(type_node.ok)
            err = self.resolve_type(type_node.err)
            return TypeValue("failure", params=[ok, err])

        if isinstance(type_node, FnType):
            params = [self.resolve_type(p) for p in type_node.params]
            return_type = self.resolve_type(type_node.return_type)
            return TypeValue("function", params=params, return_type=return_type)

        if isinstance(type_node, TupleType):
            elements = [self.resolve_type(e) for e in type_node.elements]
            return TypeValue("tuple", params=elements)

        if isinstance(type_node, ArrayType):
            inner = self.resolve_type(type_node.inner)
            return TypeValue("array", params=[inner])

        return UNKNOWN
