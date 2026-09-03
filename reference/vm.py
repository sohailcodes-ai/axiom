from dataclasses import dataclass, field
from typing import List, Any, Dict, Optional
from bytecode import *
from errors import RuntimeError_


@dataclass
class StackFrame:
    function: Function
    locals: List[Any]
    ip: int = 0
    stack: List[Any] = field(default_factory=list)

    def push(self, value: Any):
        self.stack.append(value)

    def pop(self) -> Any:
        return self.stack.pop()

    def peek(self) -> Any:
        return self.stack[-1]


class VM:
    def __init__(self, program: Program):
        self.program = program
        self.frames: List[StackFrame] = []
        self.globals: List[Any] = [None] * len(program.global_names)
        self.output: List[str] = []
        self.errors: List[RuntimeError_] = []

    def run(self) -> int:
        main = None
        for func in self.program.functions:
            if func.is_main:
                main = func
                break

        if main is None:
            self.errors.append(RuntimeError_("no main function found"))
            return 1

        frame = StackFrame(main, [None] * main.locals_count)
        self.frames.append(frame)

        try:
            while self.frames:
                self.execute_instruction()
        except RuntimeError as e:
            self.errors.append(RuntimeError_(str(e)))
            return 1

        return 0

    def current_frame(self) -> StackFrame:
        return self.frames[-1]

    def execute_instruction(self):
        frame = self.current_frame()
        if frame.ip >= len(frame.function.instructions):
            self.pop_frame()
            return

        instr = frame.function.instructions[frame.ip]
        frame.ip += 1

        if instr.opcode == Opcode.HALT:
            self.frames.clear()
            return

        if instr.opcode == Opcode.NOP:
            return

        if instr.opcode == Opcode.LOAD_CONST:
            frame.push(frame.function.constants[instr.operand])
            return

        if instr.opcode == Opcode.LOAD_TRUE:
            frame.push(True)
            return

        if instr.opcode == Opcode.LOAD_FALSE:
            frame.push(False)
            return

        if instr.opcode == Opcode.LOAD_UNIT:
            frame.push(None)
            return

        if instr.opcode == Opcode.LOAD_LOCAL:
            frame.push(frame.locals[instr.operand])
            return

        if instr.opcode == Opcode.STORE_LOCAL:
            frame.locals[instr.operand] = frame.pop()
            return

        if instr.opcode == Opcode.LOAD_GLOBAL:
            frame.push(self.globals[instr.operand])
            return

        if instr.opcode == Opcode.STORE_GLOBAL:
            self.globals[instr.operand] = frame.pop()
            return

        if instr.opcode == Opcode.POP:
            frame.pop()
            return

        if instr.opcode == Opcode.DUP:
            frame.push(frame.peek())
            return

        if instr.opcode == Opcode.ADD:
            right = frame.pop()
            left = frame.pop()
            if isinstance(left, str) and isinstance(right, str):
                frame.push(left + right)
            else:
                frame.push(left + right)
            return

        if instr.opcode == Opcode.SUB:
            right = frame.pop()
            left = frame.pop()
            frame.push(left - right)
            return

        if instr.opcode == Opcode.MUL:
            right = frame.pop()
            left = frame.pop()
            frame.push(left * right)
            return

        if instr.opcode == Opcode.DIV:
            right = frame.pop()
            left = frame.pop()
            if right == 0:
                raise RuntimeError("division by zero")
            frame.push(left // right)
            return

        if instr.opcode == Opcode.MOD:
            right = frame.pop()
            left = frame.pop()
            frame.push(left % right)
            return

        if instr.opcode == Opcode.NEG:
            value = frame.pop()
            frame.push(-value)
            return

        if instr.opcode == Opcode.EQ:
            right = frame.pop()
            left = frame.pop()
            frame.push(left == right)
            return

        if instr.opcode == Opcode.NEQ:
            right = frame.pop()
            left = frame.pop()
            frame.push(left != right)
            return

        if instr.opcode == Opcode.LT:
            right = frame.pop()
            left = frame.pop()
            frame.push(left < right)
            return

        if instr.opcode == Opcode.GT:
            right = frame.pop()
            left = frame.pop()
            frame.push(left > right)
            return

        if instr.opcode == Opcode.LTE:
            right = frame.pop()
            left = frame.pop()
            frame.push(left <= right)
            return

        if instr.opcode == Opcode.GTE:
            right = frame.pop()
            left = frame.pop()
            frame.push(left >= right)
            return

        if instr.opcode == Opcode.AND:
            right = frame.pop()
            left = frame.pop()
            frame.push(left and right)
            return

        if instr.opcode == Opcode.OR:
            right = frame.pop()
            left = frame.pop()
            frame.push(left or right)
            return

        if instr.opcode == Opcode.NOT:
            value = frame.pop()
            frame.push(not value)
            return

        if instr.opcode == Opcode.JMP:
            frame.ip = instr.operand
            return

        if instr.opcode == Opcode.JMP_IF_FALSE:
            value = frame.pop()
            if not value:
                frame.ip = instr.operand
            return

        if instr.opcode == Opcode.JMP_IF_TRUE:
            value = frame.pop()
            if value:
                frame.ip = instr.operand
            return

        if instr.opcode == Opcode.CALL_FUNC:
            func_name = instr.operand

            func = None
            for f in self.program.functions:
                if f.name == func_name:
                    func = f
                    break

            if func is None:
                raise RuntimeError(f"undefined function: {func_name}")

            arg_count = len(func.params)
            args = []
            for _ in range(arg_count):
                args.append(frame.pop())
            args.reverse()

            new_locals = [None] * func.locals_count
            for i, arg in enumerate(args):
                if i < len(new_locals):
                    new_locals[i] = arg
            new_frame = StackFrame(func, new_locals)
            self.frames.append(new_frame)
            return

        if instr.opcode == Opcode.RETURN:
            value = frame.pop()
            self.frames.pop()
            if self.frames:
                self.current_frame().push(value)
            return

        if instr.opcode == Opcode.PRINT:
            value = frame.pop()
            if value is None:
                self.output.append("()")
            else:
                self.output.append(str(value))
            frame.push(None)
            return

        if instr.opcode == Opcode.MAKE_SOME:
            value = frame.pop()
            frame.push(("some", value))
            return

        if instr.opcode == Opcode.MAKE_NONE:
            frame.push(("none", None))
            return

        if instr.opcode == Opcode.UNWRAP:
            value = frame.pop()
            if isinstance(value, tuple) and value[0] == "some":
                frame.push(value[1])
            elif isinstance(value, tuple) and value[0] == "none":
                raise RuntimeError("unwrap on None")
            else:
                frame.push(value)
            return

        if instr.opcode == Opcode.IS_SOME:
            value = frame.pop()
            frame.push(isinstance(value, tuple) and value[0] == "some")
            return

        if instr.opcode == Opcode.MAKE_STRUCT:
            name, fields = instr.operand
            values = {}
            for field in reversed(fields):
                values[field] = frame.pop()
            frame.push(("struct", name, values))
            return

        if instr.opcode == Opcode.GET_FIELD:
            obj = frame.pop()
            if isinstance(obj, tuple) and obj[0] == "struct":
                frame.push(obj[2].get(instr.operand))
            else:
                raise RuntimeError("not a struct")
            return

        if instr.opcode == Opcode.SET_FIELD:
            value = frame.pop()
            obj = frame.pop()
            if isinstance(obj, tuple) and obj[0] == "struct":
                obj[2][instr.operand] = value
                frame.push(obj)
            else:
                raise RuntimeError("not a struct")
            return

        if instr.opcode == Opcode.MAKE_VARIANT:
            name, idx = instr.operand
            value = frame.pop()
            frame.push(("variant", name, idx, value))
            return

        if instr.opcode == Opcode.GET_VARIANT:
            value = frame.pop()
            if isinstance(value, tuple) and value[0] == "variant":
                frame.push(value[3])
            else:
                raise RuntimeError("not a variant")
            return

        if instr.opcode == Opcode.CHECK_VARIANT:
            value = frame.pop()
            name, idx = instr.operand
            frame.push(isinstance(value, tuple) and value[0] == "variant" and value[1] == name and value[2] == idx)
            return

        raise RuntimeError(f"unknown opcode: {instr.opcode}")
