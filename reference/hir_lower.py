from ast_nodes import *
from hir import *
from errors import CodegenError
from tokens import Span


class HIRLowerer:
    def __init__(self, filename: str = "<input>"):
        self.filename = filename
        self.errors: List[CodegenError] = []

    def lower_program(self, program: Program) -> HIRProgram:
        functions = []
        structs = {}
        enums = {}
        imports = []

        for decl in program.declarations:
            if isinstance(decl, FnDecl):
                functions.append(self.lower_fn(decl))
            elif isinstance(decl, StructDecl):
                fields = {}
                for field in decl.fields:
                    fields[field.name] = self.type_to_str(field.type)
                structs[decl.name] = fields
            elif isinstance(decl, EnumDecl):
                variants = {}
                for variant in decl.variants:
                    variants[variant.name] = [self.type_to_str(t) for t in variant.fields]
                enums[decl.name] = variants
            elif isinstance(decl, ImportDecl):
                imports.append("::".join(decl.path))
            elif isinstance(decl, ImplBlock):
                for method in decl.methods:
                    functions.append(self.lower_fn(method))

        return HIRProgram(functions, structs, enums, imports, Span(1, 1))

    def lower_fn(self, decl: FnDecl) -> HIRFunction:
        body = self.lower_block(decl.body)
        return HIRFunction(decl.name, [p.name for p in decl.params], body, decl.span)

    def lower_block(self, block: Block) -> HIRBlock:
        stmts = []
        for stmt in block.stmts:
            lowered = self.lower_stmt(stmt)
            if lowered:
                stmts.append(lowered)
        result = self.lower_expr(block.result) if block.result else None
        return HIRBlock(stmts, result, block.span)

    def lower_stmt(self, stmt: Stmt) -> Optional[HIRStmt]:
        if isinstance(stmt, LetStmt):
            value = self.lower_expr(stmt.value)
            return HIRStmt("let", stmt.name, value, stmt.span)

        if isinstance(stmt, ExprStmt):
            expr = self.lower_expr(stmt.expr)
            return HIRStmt("expr", value=expr, span=stmt.span)

        if isinstance(stmt, ReturnStmt):
            if stmt.value:
                expr = self.lower_expr(stmt.value)
                return HIRStmt("expr", value=HIRExpr("return", expr, span=stmt.span), span=stmt.span)
            return HIRStmt("expr", value=HIRExpr("return", span=stmt.span), span=stmt.span)

        if isinstance(stmt, BreakStmt):
            return HIRStmt("expr", value=HIRExpr("break", span=stmt.span), span=stmt.span)

        if isinstance(stmt, ContinueStmt):
            return HIRStmt("expr", value=HIRExpr("continue", span=stmt.span), span=stmt.span)

        return None

    def lower_expr(self, expr: Optional[Expr]) -> Optional[HIRExpr]:
        if expr is None:
            return None

        if isinstance(expr, IntLit):
            return HIRExpr("int", int(expr.value), span=expr.span)

        if isinstance(expr, FloatLit):
            return HIRExpr("float", float(expr.value), span=expr.span)

        if isinstance(expr, StringLit):
            return HIRExpr("string", expr.value, span=expr.span)

        if isinstance(expr, BoolLit):
            return HIRExpr("bool", expr.value, span=expr.span)

        if isinstance(expr, UnitLit):
            return HIRExpr("unit", span=expr.span)

        if isinstance(expr, Ident):
            return HIRExpr("ident", expr.name, span=expr.span)

        if isinstance(expr, BinaryOp):
            left = self.lower_expr(expr.left)
            right = self.lower_expr(expr.right)
            return HIRExpr("binary", left=left, right=right, op=expr.op, span=expr.span)

        if isinstance(expr, UnaryOp):
            operand = self.lower_expr(expr.operand)
            return HIRExpr("unary", operand, op=expr.op, span=expr.span)

        if isinstance(expr, Call):
            callee = self.lower_expr(expr.callee)
            args = [self.lower_expr(a) for a in expr.args]
            return HIRExpr("call", callee=callee, args=args, span=expr.span)

        if isinstance(expr, IfExpr):
            condition = self.lower_expr(expr.condition)
            then_block = self.lower_block(expr.then_block)
            else_block = None
            if expr.else_block:
                if isinstance(expr.else_block, Block):
                    else_block = self.lower_block(expr.else_block)
                else:
                    else_block = HIRBlock([], self.lower_expr(expr.else_block), expr.else_block.span)
            return HIRExpr("if", condition=condition, then_block=then_block, else_block=else_block, span=expr.span)

        if isinstance(expr, WhileExpr):
            condition = self.lower_expr(expr.condition)
            body = self.lower_block(expr.body)
            return HIRExpr("while", condition=condition, body=body, span=expr.span)

        if isinstance(expr, ForExpr):
            var = HIRExpr("ident", expr.var, span=expr.span)
            iterable = self.lower_expr(expr.iterable)
            body = self.lower_block(expr.body)
            return HIRExpr("for", left=var, right=iterable, body=body, span=expr.span)

        if isinstance(expr, MatchExpr):
            subject = self.lower_expr(expr.subject)
            arms = []
            for arm in expr.arms:
                pattern = self.lower_pattern(arm.pattern)
                body = self.lower_expr(arm.body)
                arms.append((pattern, body))
            return HIRExpr("match", subject=subject, arms=arms, span=expr.span)

        if isinstance(expr, ReturnExpr):
            value = self.lower_expr(expr.value) if expr.value else None
            return HIRExpr("return", value, span=expr.span)

        if isinstance(expr, BreakExpr):
            return HIRExpr("break", span=expr.span)

        if isinstance(expr, ContinueExpr):
            return HIRExpr("continue", span=expr.span)

        if isinstance(expr, BlockExpr):
            block = self.lower_block(expr.block)
            return HIRExpr("block", block=block, span=expr.span)

        if isinstance(expr, StructConstruct):
            fields = {k: self.lower_expr(v) for k, v in expr.fields}
            return HIRExpr("struct_construct", expr.name, fields=fields, span=expr.span)

        if isinstance(expr, TupleConstruct):
            elements = [self.lower_expr(e) for e in expr.elements]
            return HIRExpr("tuple", elements, span=expr.span)

        if isinstance(expr, MaybeSome):
            inner = self.lower_expr(expr.value)
            return HIRExpr("maybe_some", inner, span=expr.span)

        if isinstance(expr, MaybeNone):
            return HIRExpr("maybe_none", span=expr.span)

        if isinstance(expr, Propagation):
            inner = self.lower_expr(expr.expr)
            return HIRExpr("propagation", inner, span=expr.span)

        if isinstance(expr, AddrOf):
            inner = self.lower_expr(expr.expr)
            return HIRExpr("addr_of", inner, span=expr.span)

        if isinstance(expr, FieldAccess):
            obj = self.lower_expr(expr.object)
            return HIRExpr("field", object=obj, field=expr.field, span=expr.span)

        if isinstance(expr, IndexAccess):
            obj = self.lower_expr(expr.object)
            index = self.lower_expr(expr.index)
            return HIRExpr("index", left=obj, right=index, span=expr.span)

        return HIRExpr("unit", span=expr.span)

    def lower_pattern(self, pattern: Pattern) -> Any:
        if isinstance(pattern, LiteralPattern):
            return ("literal", pattern.value)
        if isinstance(pattern, IdentPattern):
            return ("ident", pattern.name)
        if isinstance(pattern, WildcardPattern):
            return ("wildcard", None)
        if isinstance(pattern, TuplePattern):
            return ("tuple", [self.lower_pattern(p) for p in pattern.elements])
        if isinstance(pattern, EnumPattern):
            return ("enum", pattern.name, pattern.inner)
        if isinstance(pattern, OrPattern):
            return ("or", [self.lower_pattern(p) for p in pattern.patterns])
        return ("wildcard", None)

    def type_to_str(self, type_node: Optional[Type]) -> str:
        if type_node is None:
            return "()"
        if isinstance(type_node, PrimitiveType):
            return type_node.name
        if isinstance(type_node, NamedType):
            return type_node.name
        if isinstance(type_node, RefType):
            return f"&{'mut ' if type_node.mutable else ''}{self.type_to_str(type_node.inner)}"
        if isinstance(type_node, MaybeType):
            return f"maybe {self.type_to_str(type_node.inner)}"
        if isinstance(type_node, FailureType):
            return f"{self.type_to_str(type_node.ok)} ! {self.type_to_str(type_node.err)}"
        if isinstance(type_node, FnType):
            params = ", ".join(self.type_to_str(p) for p in type_node.params)
            return f"fn({params}) -> {self.type_to_str(type_node.return_type)}"
        if isinstance(type_node, TupleType):
            return f"({', '.join(self.type_to_str(e) for e in type_node.elements)})"
        if isinstance(type_node, ArrayType):
            return f"[{self.type_to_str(type_node.inner)}; {type_node.size}]"
        return "unknown"
