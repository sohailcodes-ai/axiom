use crate::errors::RuntimeError;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
    Some(Box<Value>),
    None,
    Struct(String, Vec<(String, Value)>),
    Enum(String, usize, Box<Value>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
            Value::Some(v) => write!(f, "Some({})", v),
            Value::None => write!(f, "None"),
            Value::Struct(name, _) => write!(f, "{}{{...}}", name),
            Value::Enum(name, _, _) => write!(f, "{}::...", name),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    LoadConst,
    LoadTrue,
    LoadFalse,
    LoadUnit,
    LoadLocal,
    StoreLocal,
    LoadGlobal,
    StoreGlobal,
    Pop,
    Dup,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    Not,
    Jmp,
    JmpIfFalse,
    JmpIfTrue,
    CallFunc,
    Return,
    Print,
    MakeSome,
    MakeNone,
    Unwrap,
    IsSome,
    MakeStruct,
    GetField,
    SetField,
    MakeVariant,
    GetVariant,
    CheckVariant,
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
pub enum Constant {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
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
pub struct Program {
    pub functions: Vec<Function>,
    pub global_names: Vec<String>,
}

struct StackFrame {
    function_idx: usize,
    locals: Vec<Value>,
    ip: usize,
    stack: Vec<Value>,
}

impl StackFrame {
    fn new(function_idx: usize, locals: Vec<Value>) -> Self {
        StackFrame {
            function_idx,
            locals,
            ip: 0,
            stack: Vec::new(),
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::Unit)
    }
}

pub struct VM {
    program: Program,
    frames: Vec<StackFrame>,
    globals: Vec<Value>,
    pub output: Vec<String>,
    pub errors: Vec<RuntimeError>,
}

impl VM {
    pub fn new(program: Program) -> Self {
        let globals = vec![Value::Unit; program.global_names.len()];
        VM {
            program,
            frames: Vec::new(),
            globals,
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn run(&mut self) -> i32 {
        let main_idx = self.program.functions.iter().position(|f| f.is_main);
        
        let main_idx = match main_idx {
            Some(idx) => idx,
            None => {
                self.errors.push(RuntimeError::new("no main function found"));
                return 1;
            }
        };
        
        let locals_count = self.program.functions[main_idx].locals_count;
        let frame = StackFrame::new(main_idx, vec![Value::Unit; locals_count]);
        self.frames.push(frame);
        
        loop {
            if self.frames.is_empty() {
                break;
            }
            
            if let Err(e) = self.execute_instruction() {
                self.errors.push(e);
                return 1;
            }
        }
        
        0
    }

    fn execute_instruction(&mut self) -> Result<(), RuntimeError> {
        let frame_idx = self.frames.len() - 1;
        let func_idx = self.frames[frame_idx].function_idx;
        let func = &self.program.functions[func_idx];
        
        if self.frames[frame_idx].ip >= func.instructions.len() {
            self.frames.pop();
            return Ok(());
        }
        
        let instr = func.instructions[self.frames[frame_idx].ip].clone();
        self.frames[frame_idx].ip += 1;
        
        match instr.opcode {
            Opcode::Halt => {
                self.frames.clear();
                return Ok(());
            }
            Opcode::Nop => {
                return Ok(());
            }
            Opcode::LoadConst => {
                let idx = instr.operand.unwrap() as usize;
                let value = self.constant_to_value(func_idx, idx);
                self.frames[frame_idx].push(value);
            }
            Opcode::LoadTrue => {
                self.frames[frame_idx].push(Value::Bool(true));
            }
            Opcode::LoadFalse => {
                self.frames[frame_idx].push(Value::Bool(false));
            }
            Opcode::LoadUnit => {
                self.frames[frame_idx].push(Value::Unit);
            }
            Opcode::LoadLocal => {
                let idx = instr.operand.unwrap() as usize;
                let value = self.frames[frame_idx].locals[idx].clone();
                self.frames[frame_idx].push(value);
            }
            Opcode::StoreLocal => {
                let idx = instr.operand.unwrap() as usize;
                let value = self.frames[frame_idx].pop();
                self.frames[frame_idx].locals[idx] = value;
            }
            Opcode::LoadGlobal => {
                let idx = instr.operand.unwrap() as usize;
                let value = self.globals[idx].clone();
                self.frames[frame_idx].push(value);
            }
            Opcode::StoreGlobal => {
                let idx = instr.operand.unwrap() as usize;
                let value = self.frames[frame_idx].pop();
                self.globals[idx] = value;
            }
            Opcode::Pop => {
                self.frames[frame_idx].pop();
            }
            Opcode::Dup => {
                let value = self.frames[frame_idx].stack.last().unwrap().clone();
                self.frames[frame_idx].push(value);
            }
            Opcode::Add => {
                let right = self.frames[frame_idx].pop();
                let left = self.frames[frame_idx].pop();
                let result = Self::add_values_static(left, right)?;
                self.frames[frame_idx].push(result);
            }
            Opcode::Sub => {
                let right = self.frames[frame_idx].pop();
                let left = Self::pop_value(self);
                let result = Self::sub_values_static(left, right)?;
                self.frames[frame_idx].push(result);
            }
            Opcode::Mul => {
                let right = self.frames[frame_idx].pop();
                let left = Self::pop_value(self);
                let result = Self::mul_values_static(left, right)?;
                self.frames[frame_idx].push(result);
            }
            Opcode::Div => {
                let right = self.frames[frame_idx].pop();
                let left = Self::pop_value(self);
                let result = Self::div_values_static(left, right)?;
                self.frames[frame_idx].push(result);
            }
            Opcode::Mod => {
                let right = Self::pop_value(self);
                let left = Self::pop_value(self);
                let result = Self::mod_values_static(left, right)?;
                self.frames[frame_idx].push(result);
            }
            Opcode::Neg => {
                let value = Self::pop_value(self);
                let result = Self::neg_value_static(value)?;
                self.frames[frame_idx].push(result);
            }
            Opcode::Eq => {
                let right = Self::pop_value(self);
                let left = Self::pop_value(self);
                self.frames[frame_idx].push(Value::Bool(Self::values_equal(&left, &right)));
            }
            Opcode::Neq => {
                let right = Self::pop_value(self);
                let left = Self::pop_value(self);
                self.frames[frame_idx].push(Value::Bool(!Self::values_equal(&left, &right)));
            }
            Opcode::Lt => {
                let right = Self::pop_value(self);
                let left = Self::pop_value(self);
                self.frames[frame_idx].push(Value::Bool(Self::values_less_than(&left, &right)?));
            }
            Opcode::Gt => {
                let right = Self::pop_value(self);
                let left = Self::pop_value(self);
                self.frames[frame_idx].push(Value::Bool(Self::values_greater_than(&left, &right)?));
            }
            Opcode::Lte => {
                let right = Self::pop_value(self);
                let left = Self::pop_value(self);
                self.frames[frame_idx].push(Value::Bool(Self::values_less_equal(&left, &right)?));
            }
            Opcode::Gte => {
                let right = Self::pop_value(self);
                let left = Self::pop_value(self);
                self.frames[frame_idx].push(Value::Bool(Self::values_greater_equal(&left, &right)?));
            }
            Opcode::And => {
                let right = Self::pop_value(self);
                let left = Self::pop_value(self);
                let result = Self::and_values_static(left, right)?;
                self.frames[frame_idx].push(result);
            }
            Opcode::Or => {
                let right = Self::pop_value(self);
                let left = Self::pop_value(self);
                let result = Self::or_values_static(left, right)?;
                self.frames[frame_idx].push(result);
            }
            Opcode::Not => {
                let value = Self::pop_value(self);
                let result = Self::not_value_static(value)?;
                self.frames[frame_idx].push(result);
            }
            Opcode::Jmp => {
                let target = instr.operand.unwrap() as usize;
                self.frames[frame_idx].ip = target;
            }
            Opcode::JmpIfFalse => {
                let value = Self::pop_value(self);
                if Self::is_false_static(&value) {
                    let target = instr.operand.unwrap() as usize;
                    self.frames[frame_idx].ip = target;
                }
            }
            Opcode::JmpIfTrue => {
                let value = Self::pop_value(self);
                if !Self::is_false_static(&value) {
                    let target = instr.operand.unwrap() as usize;
                    self.frames[frame_idx].ip = target;
                }
            }
            Opcode::CallFunc => {
                let func_name_val = Self::pop_value(self);
                if let Value::String(name) = func_name_val {
                    let func = self.program.functions.iter().find(|f| f.name == name).cloned();
                    
                    if let Some(func) = func {
                        let arg_count = func.param_count;
                        let mut args = Vec::new();
                        for _ in 0..arg_count {
                            args.push(Self::pop_value(self));
                        }
                        args.reverse();
                        
                        let mut locals = vec![Value::Unit; func.locals_count];
                        for (i, arg) in args.into_iter().enumerate() {
                            if i < locals.len() {
                                locals[i] = arg;
                            }
                        }
                        
                        let func_idx = self.program.functions.iter().position(|f| f.name == name).unwrap();
                        let new_frame = StackFrame::new(func_idx, locals);
                        self.frames.push(new_frame);
                    } else {
                        return Err(RuntimeError::new(&format!("undefined function: {}", name)));
                    }
                } else {
                    return Err(RuntimeError::new("call requires function name"));
                }
            }
            Opcode::Return => {
                let value = Self::pop_value(self);
                self.frames.pop();
                if let Some(frame) = self.frames.last_mut() {
                    frame.push(value);
                }
            }
            Opcode::Print => {
                let value = Self::pop_value(self);
                self.output.push(value.to_string());
                self.frames[frame_idx].push(Value::Unit);
            }
            Opcode::MakeSome => {
                let value = Self::pop_value(self);
                self.frames[frame_idx].push(Value::Some(Box::new(value)));
            }
            Opcode::MakeNone => {
                self.frames[frame_idx].push(Value::None);
            }
            Opcode::Unwrap => {
                let value = Self::pop_value(self);
                match value {
                    Value::Some(inner) => self.frames[frame_idx].push(*inner),
                    Value::None => return Err(RuntimeError::new("unwrap on None")),
                    other => self.frames[frame_idx].push(other),
                }
            }
            Opcode::IsSome => {
                let value = Self::pop_value(self);
                match value {
                    Value::Some(_) => self.frames[frame_idx].push(Value::Bool(true)),
                    _ => self.frames[frame_idx].push(Value::Bool(false)),
                }
            }
            Opcode::MakeStruct => {
                self.frames[frame_idx].push(Value::Struct(String::new(), Vec::new()));
            }
            Opcode::GetField => {
                let value = Self::pop_value(self);
                self.frames[frame_idx].push(value);
            }
            Opcode::SetField => {
                let value = Self::pop_value(self);
                let _obj = Self::pop_value(self);
                self.frames[frame_idx].push(value);
            }
            Opcode::MakeVariant => {
                let value = Self::pop_value(self);
                self.frames[frame_idx].push(Value::Enum(String::new(), 0, Box::new(value)));
            }
            Opcode::GetVariant => {
                let value = Self::pop_value(self);
                match value {
                    Value::Enum(_, _, inner) => self.frames[frame_idx].push(*inner),
                    _ => return Err(RuntimeError::new("not a variant")),
                }
            }
            Opcode::CheckVariant => {
                let value = Self::pop_value(self);
                match value {
                    Value::Enum(_, idx, _) => {
                        self.frames[frame_idx].push(Value::Bool(idx == instr.operand.unwrap() as usize))
                    }
                    _ => self.frames[frame_idx].push(Value::Bool(false)),
                }
            }
        }
        
        Ok(())
    }

    fn pop_value(&mut self) -> Value {
        let frame_idx = self.frames.len() - 1;
        self.frames[frame_idx].pop()
    }

    fn constant_to_value(&self, func_idx: usize, idx: usize) -> Value {
        match &self.program.functions[func_idx].constants[idx] {
            Constant::Integer(n) => Value::Integer(*n),
            Constant::Float(n) => Value::Float(*n),
            Constant::String(s) => Value::String(s.clone()),
            Constant::Bool(b) => Value::Bool(*b),
            Constant::Unit => Value::Unit,
        }
    }

    fn add_values_static(left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(RuntimeError::new("invalid operands for addition")),
        }
    }

    fn sub_values_static(left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            _ => Err(RuntimeError::new("invalid operands for subtraction")),
        }
    }

    fn mul_values_static(left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            _ => Err(RuntimeError::new("invalid operands for multiplication")),
        }
    }

    fn div_values_static(left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err(RuntimeError::new("division by zero"))
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            _ => Err(RuntimeError::new("invalid operands for division")),
        }
    }

    fn mod_values_static(left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err(RuntimeError::new("division by zero"))
                } else {
                    Ok(Value::Integer(a % b))
                }
            }
            _ => Err(RuntimeError::new("invalid operands for modulo")),
        }
    }

    fn neg_value_static(value: Value) -> Result<Value, RuntimeError> {
        match value {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err(RuntimeError::new("invalid operand for negation")),
        }
    }

    fn values_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::None, Value::None) => true,
            (Value::Some(a), Value::Some(b)) => Self::values_equal(a, b),
            _ => false,
        }
    }

    fn values_less_than(left: &Value, right: &Value) -> Result<bool, RuntimeError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
            (Value::Float(a), Value::Float(b)) => Ok(a < b),
            _ => Err(RuntimeError::new("invalid operands for comparison")),
        }
    }

    fn values_greater_than(left: &Value, right: &Value) -> Result<bool, RuntimeError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
            (Value::Float(a), Value::Float(b)) => Ok(a > b),
            _ => Err(RuntimeError::new("invalid operands for comparison")),
        }
    }

    fn values_less_equal(left: &Value, right: &Value) -> Result<bool, RuntimeError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a <= b),
            (Value::Float(a), Value::Float(b)) => Ok(a <= b),
            _ => Err(RuntimeError::new("invalid operands for comparison")),
        }
    }

    fn values_greater_equal(left: &Value, right: &Value) -> Result<bool, RuntimeError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a >= b),
            (Value::Float(a), Value::Float(b)) => Ok(a >= b),
            _ => Err(RuntimeError::new("invalid operands for comparison")),
        }
    }

    fn and_values_static(left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            _ => Err(RuntimeError::new("invalid operands for logical AND")),
        }
    }

    fn or_values_static(left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            _ => Err(RuntimeError::new("invalid operands for logical OR")),
        }
    }

    fn not_value_static(value: Value) -> Result<Value, RuntimeError> {
        match value {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(RuntimeError::new("invalid operand for logical NOT")),
        }
    }

    fn is_false_static(value: &Value) -> bool {
        match value {
            Value::Bool(false) => true,
            Value::None => true,
            Value::Unit => true,
            Value::Integer(0) => true,
            Value::Float(0.0) => true,
            Value::String(s) => s.is_empty(),
            _ => false,
        }
    }
}
