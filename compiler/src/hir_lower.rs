use std::collections::HashMap;
use crate::ast::*;
use crate::hir::*;
use crate::errors::CompileError;
use crate::domain_analysis::DomainKind;

pub struct HIRLowerer {
    filename: String,
    scopes: Vec<HashMap<String, usize>>,
    local_count: usize,
    function_domains: HashMap<String, DomainKind>,
    current_domain: Option<DomainKind>,
    pub errors: Vec<CompileError>,
}

impl HIRLowerer {
    pub fn new(filename: &str) -> Self {
        HIRLowerer {
            filename: filename.to_string(),
            scopes: vec![HashMap::new()],
            local_count: 0,
            function_domains: HashMap::new(),
            current_domain: None,
            errors: Vec::new(),
        }
    }

    pub fn set_domain_map(&mut self, map: HashMap<String, DomainKind>) {
        self.function_domains = map;
    }

    pub fn lower_program(&mut self, program: &Program) -> HIRProgram {
        let mut functions = Vec::new();
        
        for item in &program.items {
            match item {
                TopLevel::Function(func) => {
                    functions.push(self.lower_function(func));
                }
                TopLevel::Domain(d) => {
                    self.lower_domain(d, &mut functions);
                }
                _ => {}
            }
        }
        
        HIRProgram { functions }
    }

    fn lower_domain(&mut self, domain: &crate::ast::DomainDef, functions: &mut Vec<HIRFunction>) {
        for item in &domain.items {
            match item {
                TopLevel::Function(func) => {
                    functions.push(self.lower_function(func));
                }
                TopLevel::Domain(d) => {
                    self.lower_domain(d, functions);
                }
                _ => {}
            }
        }
    }

    fn lower_function(&mut self, func: &FunctionDef) -> HIRFunction {
        self.push_scope();
        self.local_count = 0;
        self.current_domain = self.function_domains.get(&func.name).copied();

        let mut param_indices = Vec::new();
        for param in &func.params {
            let idx = self.define_local(&param.name);
            param_indices.push(idx);
        }
        
        let body = self.lower_block(&func.body);
        
        let locals_count = self.local_count;
        self.pop_scope();
        self.current_domain = None;

        HIRFunction {
            name: func.name.clone(),
            params: param_indices.iter().map(|i| i.to_string()).collect(),
            param_count: func.params.len(),
            locals_count,
            body,
            is_main: func.name == "main",
        }
    }

    fn lower_block(&mut self, block: &Block) -> Vec<HIRStmt> {
        self.push_scope();
        
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            stmts.push(self.lower_stmt(stmt));
        }
        
        if let Some(result) = &block.result {
            stmts.push(HIRStmt::Expr(self.lower_expr(result)));
        }
        
        self.pop_scope();
        
        stmts
    }

    fn lower_stmt(&mut self, stmt: &Stmt) -> HIRStmt {
        match stmt {
            Stmt::Let { name, ty: _, value, mutable: _ } => {
                let idx = self.define_local(name);
                let value = self.lower_expr(value);
                HIRStmt::Let(idx, value)
            }
            Stmt::Expr(expr) => {
                HIRStmt::Expr(self.lower_expr(expr))
            }
            Stmt::Return(value) => {
                if let Some(expr) = value {
                    HIRStmt::Return(Some(self.lower_expr(expr)))
                } else {
                    HIRStmt::Return(None)
                }
            }
            Stmt::Print(expr) => {
                HIRStmt::Print(self.lower_expr(expr))
            }
            Stmt::While { condition, body } => {
                HIRStmt::While {
                    condition: self.lower_expr(condition),
                    body: self.lower_block(body),
                }
            }
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> HIRExpr {
        match expr {
            Expr::Integer(n) => HIRExpr::Integer(*n),
            Expr::Float(n) => HIRExpr::Float(*n),
            Expr::String(s) => HIRExpr::String(s.clone()),
            Expr::Bool(b) => HIRExpr::Bool(*b),
            Expr::NoneValue => HIRExpr::Unit,
            Expr::Identifier(name) => {
                if let Some(idx) = self.lookup(name) {
                    HIRExpr::LoadLocal(idx)
                } else {
                    HIRExpr::Unit
                }
            }
            Expr::BinaryOp { op, left, right } => {
                HIRExpr::BinaryOp {
                    op: self.lower_binop(op),
                    left: Box::new(self.lower_expr(left)),
                    right: Box::new(self.lower_expr(right)),
                }
            }
            Expr::UnaryOp { op, expr } => {
                HIRExpr::UnaryOp {
                    op: self.lower_unaryop(op),
                    expr: Box::new(self.lower_expr(expr)),
                }
            }
            Expr::FunctionCall { name, args } => {
                if let Expr::Identifier(func_name) = name.as_ref() {
                    let lowered_args = args.iter().map(|a| self.lower_expr(a)).collect();
                    
                    if func_name == "print" {
                        if let Some(arg) = args.first() {
                            return HIRExpr::Print(Box::new(self.lower_expr(arg)));
                        }
                    }

                    let callee_domain = self.function_domains.get(func_name).copied();
                    if let (Some(from), Some(to)) = (self.current_domain, callee_domain) {
                        if from != to {
                            return HIRExpr::CrossDomainCall {
                                func: func_name.clone(),
                                args: lowered_args,
                                from_domain: from.to_string(),
                                to_domain: to.to_string(),
                            };
                        }
                    }
                    
                    HIRExpr::Call {
                        func: func_name.clone(),
                        args: lowered_args,
                    }
                } else {
                    HIRExpr::Unit
                }
            }
            Expr::If { condition, then_body, else_body } => {
                HIRExpr::If {
                    condition: Box::new(self.lower_expr(condition)),
                    then_body: self.lower_block(then_body),
                    else_body: else_body.as_ref().map(|b| self.lower_block(b)),
                }
            }
            Expr::While { condition, body } => {
                HIRExpr::While {
                    condition: Box::new(self.lower_expr(condition)),
                    body: self.lower_block(body),
                }
            }
            Expr::Block(block) => {
                HIRExpr::Unit
            }
            Expr::Assignment { name, value } => {
                if let Expr::Identifier(var_name) = name.as_ref() {
                    if let Some(idx) = self.lookup(var_name) {
                        let value = self.lower_expr(value);
                        HIRExpr::StoreLocal(idx, Box::new(HIRStmt::Assign(idx, value)))
                    } else {
                        HIRExpr::Unit
                    }
                } else {
                    HIRExpr::Unit
                }
            }
            Expr::FieldAccess { object, field } => {
                HIRExpr::Unit
            }
            Expr::MethodCall { object, method, args } => {
                HIRExpr::Unit
            }
            Expr::ArrayLiteral(elements) => {
                HIRExpr::Unit
            }
            Expr::ArrayAccess { array, index } => {
                HIRExpr::Unit
            }
            Expr::TupleLiteral(elements) => {
                HIRExpr::Unit
            }
            Expr::SomeValue(value) => {
                self.lower_expr(value)
            }
            Expr::StructLiteral { name, fields } => {
                HIRExpr::Unit
            }
            Expr::QualifiedName { module, name } => {
                HIRExpr::Call {
                    func: format!("{}::{}", module, name),
                    args: Vec::new(),
                }
            }
        }
    }

    fn lower_binop(&self, op: &BinOp) -> HIRBinOp {
        match op {
            BinOp::Add => HIRBinOp::Add,
            BinOp::Sub => HIRBinOp::Sub,
            BinOp::Mul => HIRBinOp::Mul,
            BinOp::Div => HIRBinOp::Div,
            BinOp::Mod => HIRBinOp::Mod,
            BinOp::Eq => HIRBinOp::Eq,
            BinOp::Neq => HIRBinOp::Neq,
            BinOp::Lt => HIRBinOp::Lt,
            BinOp::Gt => HIRBinOp::Gt,
            BinOp::Lte => HIRBinOp::Lte,
            BinOp::Gte => HIRBinOp::Gte,
            BinOp::And => HIRBinOp::And,
            BinOp::Or => HIRBinOp::Or,
        }
    }

    fn lower_unaryop(&self, op: &UnaryOp) -> HIRUnaryOp {
        match op {
            UnaryOp::Neg => HIRUnaryOp::Neg,
            UnaryOp::Not => HIRUnaryOp::Not,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_local(&mut self, name: &str) -> usize {
        let idx = self.local_count;
        self.local_count += 1;
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), idx);
        }
        idx
    }

    fn lookup(&self, name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(&idx) = scope.get(name) {
                return Some(idx);
            }
        }
        None
    }
}
