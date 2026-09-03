use crate::errors::CompileError;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VMProgram {
    pub functions: Vec<VMFunction>,
    pub global_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VMFunction {
    pub name: String,
    pub instructions: Vec<VMInstruction>,
    pub constants: Vec<VMConstant>,
    pub locals_count: usize,
    pub param_count: usize,
    pub is_main: bool,
}

#[derive(Debug, Clone)]
pub struct VMInstruction {
    pub opcode: VMOpcode,
    pub operand: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VMOpcode {
    LoadConst,
    LoadTrue,
    LoadFalse,
    LoadUnit,
    LoadLocal,
    StoreLocal,
    Pop,
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
    Halt,
}

#[derive(Debug, Clone)]
pub enum VMConstant {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}

struct StackFrame {
    function_idx: usize,
    locals: Vec<Value>,
    ip: usize,
    stack: Vec<Value>,
}

pub struct VM {
    program: VMProgram,
    frames: Vec<StackFrame>,
    pub output: Vec<String>,
    pub errors: Vec<String>,
}

impl VM {
    pub fn new(program: VMProgram) -> Self {
        VM {
            program,
            frames: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn run(&mut self) -> i32 {
        let main_idx = self.program.functions.iter().position(|f| f.is_main);
        
        let main_idx = match main_idx {
            Some(idx) => idx,
            None => {
                self.errors.push("no main function found".to_string());
                return 1;
            }
        };
        
        let locals_count = self.program.functions[main_idx].locals_count;
        let frame = StackFrame {
            function_idx: main_idx,
            locals: vec![Value::Unit; locals_count],
            ip: 0,
            stack: Vec::new(),
        };
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

    fn execute_instruction(&mut self) -> Result<(), String> {
        let frame_idx = self.frames.len() - 1;
        let func_idx = self.frames[frame_idx].function_idx;
        
        if self.frames[frame_idx].ip >= self.program.functions[func_idx].instructions.len() {
            self.frames.pop();
            return Ok(());
        }
        
        let instr = self.program.functions[func_idx].instructions[self.frames[frame_idx].ip].clone();
        self.frames[frame_idx].ip += 1;
        
        match instr.opcode {
            VMOpcode::Halt => {
                self.frames.clear();
                return Ok(());
            }
            VMOpcode::LoadConst => {
                let idx = instr.operand.unwrap() as usize;
                let value = self.get_constant(func_idx, idx);
                self.frames[frame_idx].stack.push(value);
            }
            VMOpcode::LoadTrue => {
                self.frames[frame_idx].stack.push(Value::Bool(true));
            }
            VMOpcode::LoadFalse => {
                self.frames[frame_idx].stack.push(Value::Bool(false));
            }
            VMOpcode::LoadUnit => {
                self.frames[frame_idx].stack.push(Value::Unit);
            }
            VMOpcode::LoadLocal => {
                let idx = instr.operand.unwrap() as usize;
                let value = self.frames[frame_idx].locals[idx].clone();
                self.frames[frame_idx].stack.push(value);
            }
            VMOpcode::StoreLocal => {
                let idx = instr.operand.unwrap() as usize;
                let value = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].locals[idx] = value;
            }
            VMOpcode::Pop => {
                self.frames[frame_idx].stack.pop();
            }
            VMOpcode::Add => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Self::add_values(left, right)?);
            }
            VMOpcode::Sub => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Self::sub_values(left, right)?);
            }
            VMOpcode::Mul => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Self::mul_values(left, right)?);
            }
            VMOpcode::Div => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Self::div_values(left, right)?);
            }
            VMOpcode::Mod => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Self::mod_values(left, right)?);
            }
            VMOpcode::Neg => {
                let value = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Self::neg_value(value)?);
            }
            VMOpcode::Eq => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Value::Bool(Self::values_equal(&left, &right)));
            }
            VMOpcode::Neq => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Value::Bool(!Self::values_equal(&left, &right)));
            }
            VMOpcode::Lt => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Value::Bool(Self::values_less_than(&left, &right)?));
            }
            VMOpcode::Gt => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Value::Bool(Self::values_greater_than(&left, &right)?));
            }
            VMOpcode::Lte => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Value::Bool(Self::values_less_equal(&left, &right)?));
            }
            VMOpcode::Gte => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Value::Bool(Self::values_greater_equal(&left, &right)?));
            }
            VMOpcode::And => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Self::and_values(left, right)?);
            }
            VMOpcode::Or => {
                let right = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let left = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Self::or_values(left, right)?);
            }
            VMOpcode::Not => {
                let value = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames[frame_idx].stack.push(Self::not_value(value)?);
            }
            VMOpcode::Jmp => {
                let target = instr.operand.unwrap() as usize;
                self.frames[frame_idx].ip = target;
            }
            VMOpcode::JmpIfFalse => {
                let value = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                if Self::is_false(&value) {
                    let target = instr.operand.unwrap() as usize;
                    self.frames[frame_idx].ip = target;
                }
            }
            VMOpcode::JmpIfTrue => {
                let value = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                if !Self::is_false(&value) {
                    let target = instr.operand.unwrap() as usize;
                    self.frames[frame_idx].ip = target;
                }
            }
            VMOpcode::CallFunc => {
                let func_name = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                if let Value::String(name) = func_name {
                    let func = self.program.functions.iter().find(|f| f.name == name).cloned();
                    
                    if let Some(func) = func {
                        let arg_count = func.param_count;
                        let mut args = Vec::new();
                        for _ in 0..arg_count {
                            args.push(self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit));
                        }
                        args.reverse();
                        
                        let mut locals = vec![Value::Unit; func.locals_count];
                        for (i, arg) in args.into_iter().enumerate() {
                            if i < locals.len() {
                                locals[i] = arg;
                            }
                        }
                        
                        let func_idx = self.program.functions.iter().position(|f| f.name == name).unwrap();
                        let new_frame = StackFrame {
                            function_idx: func_idx,
                            locals,
                            ip: 0,
                            stack: Vec::new(),
                        };
                        self.frames.push(new_frame);
                    } else {
                        return Err(format!("undefined function: {}", name));
                    }
                } else {
                    return Err("call requires function name".to_string());
                }
            }
            VMOpcode::Return => {
                let value = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames.pop();
                if let Some(frame) = self.frames.last_mut() {
                    frame.stack.push(value);
                }
            }
            VMOpcode::Print => {
                let value = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.output.push(value.to_string());
                self.frames[frame_idx].stack.push(Value::Unit);
            }
        }
        
        Ok(())
    }

    fn get_constant(&self, func_idx: usize, idx: usize) -> Value {
        match &self.program.functions[func_idx].constants[idx] {
            VMConstant::Integer(n) => Value::Integer(*n),
            VMConstant::Float(n) => Value::Float(*n),
            VMConstant::String(s) => Value::String(s.clone()),
            VMConstant::Bool(b) => Value::Bool(*b),
            VMConstant::Unit => Value::Unit,
        }
    }

    fn add_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err("invalid operands for addition".to_string()),
        }
    }

    fn sub_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            _ => Err("invalid operands for subtraction".to_string()),
        }
    }

    fn mul_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            _ => Err("invalid operands for multiplication".to_string()),
        }
    }

    fn div_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            _ => Err("invalid operands for division".to_string()),
        }
    }

    fn mod_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(Value::Integer(a % b))
                }
            }
            _ => Err("invalid operands for modulo".to_string()),
        }
    }

    fn neg_value(value: Value) -> Result<Value, String> {
        match value {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err("invalid operand for negation".to_string()),
        }
    }

    fn values_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            _ => false,
        }
    }

    fn values_less_than(left: &Value, right: &Value) -> Result<bool, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
            (Value::Float(a), Value::Float(b)) => Ok(a < b),
            _ => Err("invalid operands for comparison".to_string()),
        }
    }

    fn values_greater_than(left: &Value, right: &Value) -> Result<bool, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
            (Value::Float(a), Value::Float(b)) => Ok(a > b),
            _ => Err("invalid operands for comparison".to_string()),
        }
    }

    fn values_less_equal(left: &Value, right: &Value) -> Result<bool, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a <= b),
            (Value::Float(a), Value::Float(b)) => Ok(a <= b),
            _ => Err("invalid operands for comparison".to_string()),
        }
    }

    fn values_greater_equal(left: &Value, right: &Value) -> Result<bool, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a >= b),
            (Value::Float(a), Value::Float(b)) => Ok(a >= b),
            _ => Err("invalid operands for comparison".to_string()),
        }
    }

    fn and_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            _ => Err("invalid operands for logical AND".to_string()),
        }
    }

    fn or_values(left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            _ => Err("invalid operands for logical OR".to_string()),
        }
    }

    fn not_value(value: Value) -> Result<Value, String> {
        match value {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err("invalid operand for logical NOT".to_string()),
        }
    }

    fn is_false(value: &Value) -> bool {
        match value {
            Value::Bool(false) => true,
            Value::Unit => true,
            Value::Integer(0) => true,
            Value::Float(0.0) => true,
            Value::String(s) => s.is_empty(),
            _ => false,
        }
    }
}
