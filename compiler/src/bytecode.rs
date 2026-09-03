#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    // Constants
    LoadConst,
    LoadTrue,
    LoadFalse,
    LoadUnit,
    
    // Local variables
    LoadLocal,
    StoreLocal,
    
    // Global variables
    LoadGlobal,
    StoreGlobal,
    
    // Stack operations
    Pop,
    Dup,
    
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    
    // Comparison
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    
    // Logical
    And,
    Or,
    Not,
    
    // Control flow
    Jmp,
    JmpIfFalse,
    JmpIfTrue,
    
    // Functions
    CallFunc,
    Return,
    
    // Built-in
    Print,
    
    // Maybe
    MakeSome,
    MakeNone,
    Unwrap,
    IsSome,
    
    // Structs
    MakeStruct,
    GetField,
    SetField,
    
    // Enums
    MakeVariant,
    GetVariant,
    CheckVariant,
    
    // Special
    Halt,
    Nop,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Option<i64>,
}

impl Instruction {
    pub fn new(opcode: Opcode, operand: Option<i64>) -> Self {
        Instruction { opcode, operand }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
    pub locals_count: usize,
    pub param_count: usize,
    pub is_main: bool,
}

#[derive(Debug, Clone)]
pub enum Constant {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub global_names: Vec<String>,
}
