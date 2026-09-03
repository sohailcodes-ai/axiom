from hir import *
from bytecode import *
from errors import CodegenError
from tokens import Span
from typing import Dict, List, Optional


class CodeGenerator:
    def __init__(self, filename: str = "<input>"):
        self.filename = filename
        self.errors: List[CodegenError] = []
        self.constants: List[Any] = []
        self.global_names: List[str] = []

    def generate(self, hir: HIRProgram) -> Program:
        functions = []
        for func in hir.functions:
            gen = FunctionGenerator(self)
            bytecode_func = gen.generate_function(func)
            functions.append(bytecode_func)

        return Program(functions, self.global_names)

    def add_constant(self, value: Any) -> int:
        if value in self.constants:
            return self.constants.index(value)
        self.constants.append(value)
        return len(self.constants) - 1

    def add_global(self, name: str) -> int:
        if name not in self.global_names:
            self.global_names.append(name)
        return self.global_names.index(name)


class FunctionGenerator:
    def __init__(self, parent: CodeGenerator):
        self.parent = parent
        self.instructions: List[Instruction] = []
        self.constants: List[Any] = []
        self.locals: Dict[str, int] = {}
        self.next_local: int = 0
        self.scopes: List[Dict[str, int]] = [{}]
        self.break_labels: List[int] = []
        self.continue_labels: List[int] = []

    def push_scope(self):
        self.scopes.append({})

    def pop_scope(self):
        self.scopes.pop()

    def define_local(self, name: str) -> int:
        idx = self.next_local
        self.next_local += 1
        self.locals[name] = idx
        self.scopes[-1][name] = idx
        return idx

    def resolve_local(self, name: str) -> Optional[int]:
        for scope in reversed(self.scopes):
            if name in scope:
                return scope[name]
        return None

    def add_constant(self, value: Any) -> int:
        if value in self.constants:
            return self.constants.index(value)
        self.constants.append(value)
        return len(self.constants) - 1

    def emit(self, opcode: Opcode, operand: Any = None):
        self.instructions.append(Instruction(opcode, operand))

    def generate_function(self, func: HIRFunction) -> Function:
        for param in func.params:
            self.define_local(param)

        self.generate_block(func.body)

        if not self.instructions or self.instructions[-1].opcode != Opcode.RETURN:
            if func.body.result is None:
                self.emit(Opcode.LOAD_UNIT)
            self.emit(Opcode.RETURN)

        return Function(
            name=func.name,
            params=func.params,
            locals_count=self.next_local,
            instructions=self.instructions,
            constants=self.constants,
            is_main=func.name == "main",
        )

    def generate_block(self, block: HIRBlock):
        self.push_scope()
        for stmt in block.stmts:
            self.generate_stmt(stmt)
        if block.result:
            self.generate_expr(block.result)
        self.pop_scope()

    def generate_stmt(self, stmt: HIRStmt):
        if stmt.kind == "let":
            if stmt.value:
                self.generate_expr(stmt.value)
            else:
                self.emit(Opcode.LOAD_UNIT)
            idx = self.define_local(stmt.name)
            self.emit(Opcode.STORE_LOCAL, idx)

        elif stmt.kind == "expr":
            if stmt.value:
                self.generate_expr(stmt.value)
                self.emit(Opcode.POP)

        elif stmt.kind == "assign":
            if stmt.value:
                self.generate_expr(stmt.value)
            idx = self.resolve_local(stmt.name)
            if idx is not None:
                self.emit(Opcode.STORE_LOCAL, idx)

    def generate_expr(self, expr: HIRExpr):
        if expr.kind == "int":
            idx = self.add_constant(expr.value)
            self.emit(Opcode.LOAD_CONST, idx)

        elif expr.kind == "float":
            idx = self.add_constant(expr.value)
            self.emit(Opcode.LOAD_CONST, idx)

        elif expr.kind == "string":
            idx = self.add_constant(expr.value)
            self.emit(Opcode.LOAD_CONST, idx)

        elif expr.kind == "bool":
            if expr.value:
                self.emit(Opcode.LOAD_TRUE)
            else:
                self.emit(Opcode.LOAD_FALSE)

        elif expr.kind == "unit":
            self.emit(Opcode.LOAD_UNIT)

        elif expr.kind == "ident":
            name = expr.value
            if name == "unreachable":
                self.emit(Opcode.HALT)
                return
            idx = self.resolve_local(name)
            if idx is not None:
                self.emit(Opcode.LOAD_LOCAL, idx)
            else:
                global_idx = self.parent.add_global(name)
                self.emit(Opcode.LOAD_GLOBAL, global_idx)

        elif expr.kind == "binary":
            if expr.op in ("=", "+=", "-=", "*=", "/=", "%="):
                self.generate_expr(expr.right)
                if isinstance(expr.left, HIRExpr) and expr.left.kind == "ident":
                    idx = self.resolve_local(expr.left.value)
                    if idx is not None:
                        if expr.op == "=":
                            self.emit(Opcode.STORE_LOCAL, idx)
                        elif expr.op == "+=":
                            self.emit(Opcode.LOAD_LOCAL, idx)
                            self.emit(Opcode.ADD)
                            self.emit(Opcode.STORE_LOCAL, idx)
                        elif expr.op == "-=":
                            self.emit(Opcode.LOAD_LOCAL, idx)
                            self.emit(Opcode.SUB)
                            self.emit(Opcode.STORE_LOCAL, idx)
                        elif expr.op == "*=":
                            self.emit(Opcode.LOAD_LOCAL, idx)
                            self.emit(Opcode.MUL)
                            self.emit(Opcode.STORE_LOCAL, idx)
                        elif expr.op == "/=":
                            self.emit(Opcode.LOAD_LOCAL, idx)
                            self.emit(Opcode.DIV)
                            self.emit(Opcode.STORE_LOCAL, idx)
                        elif expr.op == "%=":
                            self.emit(Opcode.LOAD_LOCAL, idx)
                            self.emit(Opcode.MOD)
                            self.emit(Opcode.STORE_LOCAL, idx)
                        return
                return

            self.generate_expr(expr.left)
            self.generate_expr(expr.right)
            ops = {
                "+": Opcode.ADD, "-": Opcode.SUB, "*": Opcode.MUL,
                "/": Opcode.DIV, "%": Opcode.MOD, "==": Opcode.EQ,
                "!=": Opcode.NEQ, "<": Opcode.LT, ">": Opcode.GT,
                "<=": Opcode.LTE, ">=": Opcode.GTE, "&&": Opcode.AND,
                "||": Opcode.OR,
            }
            self.emit(ops.get(expr.op, Opcode.ADD))

        elif expr.kind == "unary":
            self.generate_expr(expr.value)
            if expr.op == "-":
                self.emit(Opcode.NEG)
            elif expr.op == "!":
                self.emit(Opcode.NOT)

        elif expr.kind == "call":
            if isinstance(expr.callee, HIRExpr) and expr.callee.kind == "ident":
                if expr.callee.value == "print":
                    for arg in (expr.args or []):
                        self.generate_expr(arg)
                    self.emit(Opcode.PRINT)
                    self.emit(Opcode.LOAD_UNIT)
                    return

                for arg in (expr.args or []):
                    self.generate_expr(arg)
                self.emit(Opcode.CALL_FUNC, expr.callee.value)
                return

            if expr.callee:
                self.generate_expr(expr.callee)
            for arg in (expr.args or []):
                self.generate_expr(arg)
            self.emit(Opcode.CALL, len(expr.args or []))

        elif expr.kind == "if":
            self.generate_expr(expr.condition)
            else_label = len(self.instructions)
            self.emit(Opcode.JMP_IF_FALSE, 0)
            self.generate_block(expr.then_block)
            if expr.else_block:
                end_label = len(self.instructions)
                self.emit(Opcode.JMP, 0)
                self.instructions[else_label].operand = len(self.instructions)
                self.generate_block(expr.else_block)
                self.instructions[end_label].operand = len(self.instructions)
            else:
                self.instructions[else_label].operand = len(self.instructions)

        elif expr.kind == "while":
            loop_start = len(self.instructions)
            self.generate_expr(expr.condition)
            exit_label = len(self.instructions)
            self.emit(Opcode.JMP_IF_FALSE, 0)
            self.break_labels.append(exit_label)
            self.continue_labels.append(loop_start)
            self.generate_block(expr.body)
            self.emit(Opcode.JMP, loop_start)
            self.instructions[exit_label].operand = len(self.instructions)
            self.break_labels.pop()
            self.continue_labels.pop()

        elif expr.kind == "for":
            iterable_idx = self.next_local
            self.next_local += 1
            self.generate_expr(expr.right)
            self.emit(Opcode.STORE_LOCAL, iterable_idx)

            var_idx = self.define_local(expr.left.value if isinstance(expr.left, HIRExpr) and expr.left.kind == "ident" else "item")

            idx_idx = self.next_local
            self.next_local += 1
            idx = self.add_constant(0)
            self.emit(Opcode.LOAD_CONST, idx)
            self.emit(Opcode.STORE_LOCAL, idx_idx)

            loop_start = len(self.instructions)
            idx_load = self.add_constant(0)
            self.emit(Opcode.LOAD_LOCAL, idx_idx)
            self.emit(Opcode.LOAD_LOCAL, iterable_idx)
            self.emit(Opcode.CALL, 1)
            self.emit(Opcode.CALL, 1)

            exit_label = len(self.instructions)
            self.emit(Opcode.JMP_IF_FALSE, 0)
            self.break_labels.append(exit_label)
            self.continue_labels.append(loop_start)

            self.generate_block(expr.body)

            self.emit(Opcode.LOAD_LOCAL, idx_idx)
            self.emit(Opcode.LOAD_CONST, self.add_constant(1))
            self.emit(Opcode.ADD)
            self.emit(Opcode.STORE_LOCAL, idx_idx)

            self.emit(Opcode.JMP, loop_start)
            self.instructions[exit_label].operand = len(self.instructions)
            self.break_labels.pop()
            self.continue_labels.pop()

        elif expr.kind == "return":
            if expr.value:
                self.generate_expr(expr.value)
            else:
                self.emit(Opcode.LOAD_UNIT)
            self.emit(Opcode.RETURN)

        elif expr.kind == "break":
            if self.break_labels:
                self.emit(Opcode.JMP, self.break_labels[-1])
            else:
                self.emit(Opcode.HALT)

        elif expr.kind == "continue":
            if self.continue_labels:
                self.emit(Opcode.JMP, self.continue_labels[-1])
            else:
                self.emit(Opcode.HALT)

        elif expr.kind == "block":
            self.generate_block(expr.block)

        elif expr.kind == "struct_construct":
            if expr.fields:
                for field_name, field_expr in expr.fields.items():
                    self.generate_expr(field_expr)
                self.emit(Opcode.MAKE_STRUCT, (expr.value, list(expr.fields.keys())))

        elif expr.kind == "maybe_some":
            if expr.value:
                self.generate_expr(expr.value)
                self.emit(Opcode.MAKE_SOME)
            else:
                self.emit(Opcode.LOAD_UNIT)
                self.emit(Opcode.MAKE_SOME)

        elif expr.kind == "maybe_none":
            self.emit(Opcode.MAKE_NONE)

        elif expr.kind == "propagation":
            if expr.value:
                self.generate_expr(expr.value)
                self.emit(Opcode.UNWRAP)

        elif expr.kind == "field":
            if expr.object:
                self.generate_expr(expr.object)
                self.emit(Opcode.GET_FIELD, expr.field)

        elif expr.kind == "addr_of":
            if expr.value:
                self.generate_expr(expr.value)

        elif expr.kind == "match":
            self.emit(Opcode.LOAD_UNIT)

        elif expr.kind == "tuple":
            if expr.value:
                for elem in expr.value:
                    self.generate_expr(elem)

        elif expr.kind == "index":
            if expr.left:
                self.generate_expr(expr.left)
            if expr.right:
                self.generate_expr(expr.right)

        else:
            self.emit(Opcode.LOAD_UNIT)
