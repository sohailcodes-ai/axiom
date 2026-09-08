#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
    Array(Vec<Value>),
    Struct(Vec<Value>),
    Maybe(Option<Box<Value>>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
            Value::Array(elements) => {
                let items: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Struct(fields) => {
                let items: Vec<String> = fields.iter().map(|e| e.to_string()).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::Maybe(Some(val)) => write!(f, "Some({})", val),
            Value::Maybe(None) => write!(f, "None"),
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
    CrossDomainCall,
    Return,
    Print,
    Halt,
    MakeStruct,
    GetField,
    MakeArray,
    ArrayGet,
}

#[derive(Debug, Clone)]
pub enum VMConstant {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}

pub struct StackFrame {
    pub function_idx: usize,
    pub locals: Vec<Value>,
    pub ip: usize,
    pub stack: Vec<Value>,
}

pub struct VM {
    pub program: VMProgram,
    pub frames: Vec<StackFrame>,
    cross_domain_stack: Vec<bool>,
    pub output: Vec<String>,
    pub errors: Vec<String>,
}

impl VM {
    pub fn new(program: VMProgram) -> Self {
        VM {
            program,
            frames: Vec::new(),
            cross_domain_stack: Vec::new(),
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

    pub fn execute_instruction(&mut self) -> Result<(), String> {
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
            VMOpcode::CrossDomainCall => {
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
                        
                        // SERIALIZE arguments
                        let mut serialized_args = Vec::new();
                        for arg in &args {
                            let bytes = Self::serialize_value(arg);
                            serialized_args.push(bytes);
                        }
                        
                        self.output.push(format!(
                            "[BOUNDARY] {} args serialized ({} bytes total)",
                            args.len(),
                            serialized_args.iter().map(|b| b.len()).sum::<usize>()
                        ));
                        
                        // DESERIALIZE arguments (simulating network transfer)
                        let mut deserialized_args = Vec::new();
                        for bytes in &serialized_args {
                            let (val, _) = Self::deserialize_value(bytes)
                                .map_err(|e| format!("deserialization error: {}", e))?;
                            deserialized_args.push(val);
                        }
                        
                        self.output.push(format!(
                            "[BOUNDARY] {} args deserialized successfully",
                            deserialized_args.len()
                        ));
                        
                        // Mark this as a cross-domain boundary
                        self.cross_domain_stack.push(true);
                        
                        // Execute target function with deserialized args
                        let mut locals = vec![Value::Unit; func.locals_count];
                        for (i, arg) in deserialized_args.into_iter().enumerate() {
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
                    return Err("cross-domain call requires function name".to_string());
                }
            }
            VMOpcode::Return => {
                let value = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.frames.pop();
                
                // Check if this is a cross-domain boundary return
                if !self.cross_domain_stack.is_empty() && *self.cross_domain_stack.last().unwrap() {
                    self.cross_domain_stack.pop();
                    
                    // SERIALIZE return value
                    let serialized = Self::serialize_value(&value);
                    self.output.push(format!(
                        "[BOUNDARY] return value serialized ({} bytes)",
                        serialized.len()
                    ));
                    
                    // DESERIALIZE return value (simulating network transfer)
                    let (deserialized, _) = Self::deserialize_value(&serialized)
                        .map_err(|e| format!("return deserialization error: {}", e))?;
                    
                    self.output.push(format!(
                        "[BOUNDARY] return value deserialized: {}",
                        deserialized
                    ));
                    
                    // Push deserialized result to caller
                    if let Some(frame) = self.frames.last_mut() {
                        frame.stack.push(deserialized);
                    }
                } else {
                    // Normal local return
                    if let Some(frame) = self.frames.last_mut() {
                        frame.stack.push(value);
                    }
                }
            }
            VMOpcode::Print => {
                let value = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                self.output.push(value.to_string());
                self.frames[frame_idx].stack.push(Value::Unit);
            }
            VMOpcode::MakeStruct => {
                let field_count = instr.operand.unwrap() as usize;
                let mut fields = Vec::new();
                for _ in 0..field_count {
                    fields.push(self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit));
                }
                fields.reverse();
                self.frames[frame_idx].stack.push(Value::Struct(fields));
            }
            VMOpcode::GetField => {
                let index = instr.operand.unwrap() as usize;
                let object = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                match object {
                    Value::Struct(fields) => {
                        if index < fields.len() {
                            self.frames[frame_idx].stack.push(fields[index].clone());
                        } else {
                            return Err(format!("struct field index {} out of bounds", index));
                        }
                    }
                    _ => return Err("GetField requires a struct value".to_string()),
                }
            }
            VMOpcode::MakeArray => {
                let count = instr.operand.unwrap() as usize;
                let mut elements = Vec::new();
                for _ in 0..count {
                    elements.push(self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit));
                }
                elements.reverse();
                self.frames[frame_idx].stack.push(Value::Array(elements));
            }
            VMOpcode::ArrayGet => {
                let index = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                let array = self.frames[frame_idx].stack.pop().unwrap_or(Value::Unit);
                match (array, index) {
                    (Value::Array(elements), Value::Integer(i)) => {
                        if i >= 0 && (i as usize) < elements.len() {
                            self.frames[frame_idx].stack.push(elements[i as usize].clone());
                        } else {
                            return Err(format!("array index {} out of bounds", i));
                        }
                    }
                    _ => return Err("ArrayGet requires an array and an integer index".to_string()),
                }
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
            (Value::Array(a), Value::Array(b)) => a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Self::values_equal(x, y)),
            (Value::Struct(a), Value::Struct(b)) => a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Self::values_equal(x, y)),
            (Value::Maybe(a), Value::Maybe(b)) => match (a, b) {
                (None, None) => true,
                (Some(x), Some(y)) => Self::values_equal(x, y),
                _ => false,
            },
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

    // AXIOM binary serialization format (tag-length-value)
    // Tag bytes: 0=Int, 1=Float, 2=Bool, 3=String, 4=Unit, 5=Array, 6=MaybeSome, 7=MaybeNone, 8=Struct
    
    fn serialize_value(value: &Value) -> Vec<u8> {
        let mut buf = Vec::new();
        Self::serialize_into(value, &mut buf);
        buf
    }
    
    fn serialize_into(value: &Value, buf: &mut Vec<u8>) {
        match value {
            Value::Integer(n) => {
                buf.push(0);
                buf.extend_from_slice(&n.to_le_bytes());
            }
            Value::Float(n) => {
                buf.push(1);
                buf.extend_from_slice(&n.to_le_bytes());
            }
            Value::Bool(b) => {
                buf.push(2);
                buf.push(if *b { 1 } else { 0 });
            }
            Value::String(s) => {
                buf.push(3);
                let bytes = s.as_bytes();
                buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                buf.extend_from_slice(bytes);
            }
            Value::Unit => {
                buf.push(4);
            }
            Value::Array(elements) => {
                buf.push(5);
                buf.extend_from_slice(&(elements.len() as u32).to_le_bytes());
                for elem in elements {
                    Self::serialize_into(elem, buf);
                }
            }
            Value::Maybe(Some(val)) => {
                buf.push(6);
                Self::serialize_into(val, buf);
            }
            Value::Maybe(None) => {
                buf.push(7);
            }
            Value::Struct(fields) => {
                buf.push(8);
                buf.extend_from_slice(&(fields.len() as u32).to_le_bytes());
                for field in fields {
                    Self::serialize_into(field, buf);
                }
            }
        }
    }
    
    fn deserialize_value(data: &[u8]) -> Result<(Value, usize), String> {
        if data.is_empty() {
            return Err("empty data".to_string());
        }
        
        let tag = data[0];
        let mut pos = 1;
        
        match tag {
            0 => {
                if data.len() < 9 {
                    return Err("truncated Int".to_string());
                }
                let n = i64::from_le_bytes(data[1..9].try_into().unwrap());
                Ok((Value::Integer(n), 9))
            }
            1 => {
                if data.len() < 9 {
                    return Err("truncated Float".to_string());
                }
                let n = f64::from_le_bytes(data[1..9].try_into().unwrap());
                Ok((Value::Float(n), 9))
            }
            2 => {
                if data.len() < 2 {
                    return Err("truncated Bool".to_string());
                }
                Ok((Value::Bool(data[1] != 0), 2))
            }
            3 => {
                if data.len() < 5 {
                    return Err("truncated String length".to_string());
                }
                let len = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
                if data.len() < 5 + len {
                    return Err("truncated String data".to_string());
                }
                let s = String::from_utf8(data[5..5 + len].to_vec())
                    .map_err(|e| format!("invalid UTF-8: {}", e))?;
                Ok((Value::String(s), 5 + len))
            }
            4 => Ok((Value::Unit, 1)),
            5 => {
                if data.len() < 5 {
                    return Err("truncated Array length".to_string());
                }
                let count = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
                pos = 5;
                let mut elements = Vec::new();
                for _ in 0..count {
                    let (val, consumed) = Self::deserialize_value(&data[pos..])?;
                    elements.push(val);
                    pos += consumed;
                }
                Ok((Value::Array(elements), pos))
            }
            6 => {
                let (val, consumed) = Self::deserialize_value(&data[1..])?;
                Ok((Value::Maybe(Some(Box::new(val))), 1 + consumed))
            }
            7 => Ok((Value::Maybe(None), 1)),
            8 => {
                if data.len() < 5 {
                    return Err("truncated Struct length".to_string());
                }
                let count = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
                pos = 5;
                let mut fields = Vec::new();
                for _ in 0..count {
                    let (val, consumed) = Self::deserialize_value(&data[pos..])?;
                    fields.push(val);
                    pos += consumed;
                }
                Ok((Value::Struct(fields), pos))
            }
            _ => Err(format!("unknown serialization tag: {}", tag)),
        }
    }
}
