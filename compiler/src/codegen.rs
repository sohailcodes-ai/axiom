use crate::hir::*;
use crate::vm::*;
use crate::errors::CompileError;

pub struct CodeGenerator {
    filename: String,
    pub errors: Vec<CompileError>,
}

impl CodeGenerator {
    pub fn new(filename: &str) -> Self {
        CodeGenerator {
            filename: filename.to_string(),
            errors: Vec::new(),
        }
    }

    pub fn generate(&mut self, program: &HIRProgram) -> VMProgram {
        let mut functions = Vec::new();
        
        for func in &program.functions {
            functions.push(self.generate_function(func));
        }
        
        VMProgram {
            functions,
            global_names: Vec::new(),
        }
    }

    fn generate_function(&self, func: &HIRFunction) -> VMFunction {
        let mut codegen = FunctionCodegen::new(func);
        codegen.generate(&func.body);
        
        VMFunction {
            name: func.name.clone(),
            instructions: codegen.instructions,
            constants: codegen.constants,
            locals_count: func.locals_count,
            param_count: func.param_count,
            is_main: func.is_main,
        }
    }
}

struct FunctionCodegen {
    instructions: Vec<VMInstruction>,
    constants: Vec<VMConstant>,
    locals_count: usize,
    param_count: usize,
    is_main: bool,
}

impl FunctionCodegen {
    fn new(func: &HIRFunction) -> Self {
        FunctionCodegen {
            instructions: Vec::new(),
            constants: Vec::new(),
            locals_count: func.locals_count,
            param_count: func.param_count,
            is_main: func.is_main,
        }
    }

    fn generate(&mut self, body: &[HIRStmt]) {
        for stmt in body {
            self.generate_stmt(stmt);
        }
        let has_return = matches!(body.last(), Some(HIRStmt::Return(_)));
        if !has_return {
            self.emit(VMOpcode::LoadUnit, None);
            self.emit(VMOpcode::Return, None);
        }
    }

    fn emit(&mut self, opcode: VMOpcode, operand: Option<i64>) {
        self.instructions.push(VMInstruction { opcode, operand });
    }

    fn emit_const(&mut self, constant: VMConstant) -> usize {
        let idx = self.constants.len();
        self.constants.push(constant);
        idx
    }

    pub fn generate_expr(&mut self, expr: &HIRExpr) {
        match expr {
            HIRExpr::Integer(n) => {
                let idx = self.emit_const(VMConstant::Integer(*n)) as i64;
                self.emit(VMOpcode::LoadConst, Some(idx));
            }
            HIRExpr::Float(n) => {
                let idx = self.emit_const(VMConstant::Float(*n)) as i64;
                self.emit(VMOpcode::LoadConst, Some(idx));
            }
            HIRExpr::String(s) => {
                let idx = self.emit_const(VMConstant::String(s.clone())) as i64;
                self.emit(VMOpcode::LoadConst, Some(idx));
            }
            HIRExpr::Bool(b) => {
                if *b {
                    self.emit(VMOpcode::LoadTrue, None);
                } else {
                    self.emit(VMOpcode::LoadFalse, None);
                }
            }
            HIRExpr::Unit => {
                self.emit(VMOpcode::LoadUnit, None);
            }
            HIRExpr::LoadLocal(idx) => {
                self.emit(VMOpcode::LoadLocal, Some(*idx as i64));
            }
            HIRExpr::StoreLocal(idx, stmt) => {
                self.generate_stmt(stmt);
                self.emit(VMOpcode::StoreLocal, Some(*idx as i64));
            }
            HIRExpr::BinaryOp { op, left, right } => {
                self.generate_expr(left);
                self.generate_expr(right);
                self.generate_binop(op);
            }
            HIRExpr::UnaryOp { op, expr } => {
                self.generate_expr(expr);
                self.generate_unaryop(op);
            }
            HIRExpr::Call { func, args } => {
                for arg in args {
                    self.generate_expr(arg);
                }
                let idx = self.emit_const(VMConstant::String(func.clone())) as i64;
                self.emit(VMOpcode::LoadConst, Some(idx));
                self.emit(VMOpcode::CallFunc, None);
            }
            HIRExpr::CrossDomainCall { func, args, .. } => {
                for arg in args {
                    self.generate_expr(arg);
                }
                let idx = self.emit_const(VMConstant::String(func.clone())) as i64;
                self.emit(VMOpcode::LoadConst, Some(idx));
                self.emit(VMOpcode::CallFunc, None);
            }
            HIRExpr::If { condition, then_body, else_body } => {
                self.generate_expr(condition);
                
                let then_start = self.instructions.len();
                self.emit(VMOpcode::JmpIfFalse, None);
                
                for stmt in then_body {
                    self.generate_stmt(stmt);
                }
                
                if let Some(else_body) = else_body {
                    let else_jmp = self.instructions.len();
                    self.emit(VMOpcode::Jmp, None);
                    
                    let else_start = self.instructions.len();
                    self.instructions[then_start].operand = Some(else_start as i64);
                    
                    for stmt in else_body {
                        self.generate_stmt(stmt);
                    }
                    
                    let end = self.instructions.len();
                    self.instructions[else_jmp].operand = Some(end as i64);
                } else {
                    let end = self.instructions.len();
                    self.instructions[then_start].operand = Some(end as i64);
                }
            }
            HIRExpr::While { condition, body } => {
                let loop_start = self.instructions.len();
                self.generate_expr(condition);
                
                let jmp_if_false = self.instructions.len();
                self.emit(VMOpcode::JmpIfFalse, None);
                
                for stmt in body {
                    self.generate_stmt(stmt);
                }
                
                self.emit(VMOpcode::Jmp, Some(loop_start as i64));
                
                let loop_end = self.instructions.len();
                self.instructions[jmp_if_false].operand = Some(loop_end as i64);
            }
            HIRExpr::Print(expr) => {
                self.generate_expr(expr);
                self.emit(VMOpcode::Print, None);
            }
            HIRExpr::Return(expr) => {
                self.generate_expr(expr);
                self.emit(VMOpcode::Return, None);
            }
        }
    }

    pub fn generate_stmt(&mut self, stmt: &HIRStmt) {
        match stmt {
            HIRStmt::Let(idx, expr) => {
                self.generate_expr(expr);
                self.emit(VMOpcode::StoreLocal, Some(*idx as i64));
            }
            HIRStmt::Assign(idx, expr) => {
                self.generate_expr(expr);
                self.emit(VMOpcode::StoreLocal, Some(*idx as i64));
            }
            HIRStmt::Expr(expr) => {
                self.generate_expr(expr);
                self.emit(VMOpcode::Pop, None);
            }
            HIRStmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.generate_expr(expr);
                } else {
                    self.emit(VMOpcode::LoadUnit, None);
                }
                self.emit(VMOpcode::Return, None);
            }
            HIRStmt::Print(expr) => {
                self.generate_expr(expr);
                self.emit(VMOpcode::Print, None);
            }
            HIRStmt::While { condition, body } => {
                let loop_start = self.instructions.len();
                self.generate_expr(condition);
                
                let jmp_if_false = self.instructions.len();
                self.emit(VMOpcode::JmpIfFalse, None);
                
                for stmt in body {
                    self.generate_stmt(stmt);
                }
                
                self.emit(VMOpcode::Jmp, Some(loop_start as i64));
                
                let loop_end = self.instructions.len();
                self.instructions[jmp_if_false].operand = Some(loop_end as i64);
            }
            HIRStmt::If { condition, then_body, else_body } => {
                self.generate_expr(condition);
                
                let then_start = self.instructions.len();
                self.emit(VMOpcode::JmpIfFalse, None);
                
                for stmt in then_body {
                    self.generate_stmt(stmt);
                }
                
                if let Some(else_body) = else_body {
                    let else_jmp = self.instructions.len();
                    self.emit(VMOpcode::Jmp, None);
                    
                    let else_start = self.instructions.len();
                    self.instructions[then_start].operand = Some(else_start as i64);
                    
                    for stmt in else_body {
                        self.generate_stmt(stmt);
                    }
                    
                    let end = self.instructions.len();
                    self.instructions[else_jmp].operand = Some(end as i64);
                } else {
                    let end = self.instructions.len();
                    self.instructions[then_start].operand = Some(end as i64);
                }
            }
        }
    }

    fn generate_binop(&mut self, op: &HIRBinOp) {
        match op {
            HIRBinOp::Add => self.emit(VMOpcode::Add, None),
            HIRBinOp::Sub => self.emit(VMOpcode::Sub, None),
            HIRBinOp::Mul => self.emit(VMOpcode::Mul, None),
            HIRBinOp::Div => self.emit(VMOpcode::Div, None),
            HIRBinOp::Mod => self.emit(VMOpcode::Mod, None),
            HIRBinOp::Eq => self.emit(VMOpcode::Eq, None),
            HIRBinOp::Neq => self.emit(VMOpcode::Neq, None),
            HIRBinOp::Lt => self.emit(VMOpcode::Lt, None),
            HIRBinOp::Gt => self.emit(VMOpcode::Gt, None),
            HIRBinOp::Lte => self.emit(VMOpcode::Lte, None),
            HIRBinOp::Gte => self.emit(VMOpcode::Gte, None),
            HIRBinOp::And => self.emit(VMOpcode::And, None),
            HIRBinOp::Or => self.emit(VMOpcode::Or, None),
        }
    }

    fn generate_unaryop(&mut self, op: &HIRUnaryOp) {
        match op {
            HIRUnaryOp::Neg => self.emit(VMOpcode::Neg, None),
            HIRUnaryOp::Not => self.emit(VMOpcode::Not, None),
        }
    }
}
