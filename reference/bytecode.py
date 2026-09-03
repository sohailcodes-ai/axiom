from enum import Enum, auto
from dataclasses import dataclass
from typing import List, Any


class Opcode(Enum):
    NOP = auto()
    HALT = auto()

    # Constants
    LOAD_CONST = auto()
    LOAD_TRUE = auto()
    LOAD_FALSE = auto()
    LOAD_UNIT = auto()

    # Variables
    LOAD_LOCAL = auto()
    STORE_LOCAL = auto()
    LOAD_GLOBAL = auto()
    STORE_GLOBAL = auto()

    # Stack
    POP = auto()
    DUP = auto()

    # Arithmetic
    ADD = auto()
    SUB = auto()
    MUL = auto()
    DIV = auto()
    MOD = auto()
    NEG = auto()

    # Comparison
    EQ = auto()
    NEQ = auto()
    LT = auto()
    GT = auto()
    LTE = auto()
    GTE = auto()

    # Logical
    AND = auto()
    OR = auto()
    NOT = auto()

    # Control flow
    JMP = auto()
    JMP_IF_FALSE = auto()
    JMP_IF_TRUE = auto()

    # Functions
    CALL = auto()
    CALL_FUNC = auto()
    RETURN = auto()

    # IO
    PRINT = auto()

    # Maybe
    MAKE_SOME = auto()
    MAKE_NONE = auto()
    UNWRAP = auto()
    IS_SOME = auto()

    # Struct
    MAKE_STRUCT = auto()
    GET_FIELD = auto()
    SET_FIELD = auto()

    # Enum
    MAKE_VARIANT = auto()
    GET_VARIANT = auto()
    CHECK_VARIANT = auto()


@dataclass
class Instruction:
    opcode: Opcode
    operand: Any = None

    def __repr__(self) -> str:
        if self.operand is not None:
            return f"{self.opcode.name}({self.operand})"
        return self.opcode.name


@dataclass
class Function:
    name: str
    params: List[str]
    locals_count: int
    instructions: List[Instruction]
    constants: List[Any]
    is_main: bool = False


@dataclass
class Program:
    functions: List[Function]
    global_names: List[str]
