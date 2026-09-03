use std::collections::HashMap;
use crate::ast::*;
use crate::errors::CompileError;

pub struct NameResolver {
    filename: String,
    scopes: Vec<HashMap<String, bool>>,
    pub errors: Vec<CompileError>,
}

impl NameResolver {
    pub fn new(filename: &str) -> Self {
        NameResolver {
            filename: filename.to_string(),
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
        }
    }

    pub fn resolve_program(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Function(func) => {
                    self.resolve_function(func);
                }
                TopLevel::Struct(s) => {
                    self.resolve_struct(s);
                }
                TopLevel::Enum(e) => {
                    self.resolve_enum(e);
                }
            }
        }
    }

    fn resolve_function(&mut self, func: &FunctionDef) {
        self.define(&func.name, true);
        
        self.push_scope();
        
        for param in &func.params {
            self.define(&param.name, true);
        }
        
        self.resolve_block(&func.body);
        
        self.pop_scope();
    }

    fn resolve_struct(&mut self, s: &StructDef) {
        self.define(&s.name, true);
    }

    fn resolve_enum(&mut self, e: &EnumDef) {
        self.define(&e.name, true);
    }

    fn resolve_block(&mut self, block: &Block) {
        self.push_scope();
        
        for stmt in &block.stmts {
            self.resolve_stmt(stmt);
        }
        
        if let Some(result) = &block.result {
            self.resolve_expr(result);
        }
        
        self.pop_scope();
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, ty: _, value, mutable: _ } => {
                self.resolve_expr(value);
                self.define(name, true);
            }
            Stmt::Expr(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.resolve_expr(expr);
                }
            }
            Stmt::Print(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_block(body);
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(_) | Expr::Float(_) | Expr::String(_) | Expr::Bool(_) | Expr::NoneValue => {}
            Expr::Identifier(name) => {
                if !self.is_defined(name) {
                    self.errors.push(CompileError::new(
                        &format!("undefined variable: {}", name),
                        &self.filename,
                        0,
                        0,
                    ));
                }
            }
            Expr::BinaryOp { op: _, left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::UnaryOp { op: _, expr } => {
                self.resolve_expr(expr);
            }
            Expr::FunctionCall { name, args } => {
                self.resolve_expr(name);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::If { condition, then_body, else_body } => {
                self.resolve_expr(condition);
                self.resolve_block(then_body);
                if let Some(else_body) = else_body {
                    self.resolve_block(else_body);
                }
            }
            Expr::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_block(body);
            }
            Expr::Block(block) => {
                self.resolve_block(block);
            }
            Expr::Assignment { name, value } => {
                self.resolve_expr(name);
                self.resolve_expr(value);
            }
            Expr::FieldAccess { object, field: _ } => {
                self.resolve_expr(object);
            }
            Expr::MethodCall { object, method: _, args } => {
                self.resolve_expr(object);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::ArrayLiteral(elements) => {
                for elem in elements {
                    self.resolve_expr(elem);
                }
            }
            Expr::ArrayAccess { array, index } => {
                self.resolve_expr(array);
                self.resolve_expr(index);
            }
            Expr::TupleLiteral(elements) => {
                for elem in elements {
                    self.resolve_expr(elem);
                }
            }
            Expr::SomeValue(value) => {
                self.resolve_expr(value);
            }
            Expr::StructLiteral { name: _, fields } => {
                for (_, value) in fields {
                    self.resolve_expr(value);
                }
            }
            Expr::QualifiedName { module: _, name: _ } => {}
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: &str, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), mutable);
        }
    }

    fn is_defined(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }
}
