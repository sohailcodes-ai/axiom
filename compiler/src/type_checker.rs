use std::collections::HashMap;
use crate::ast::*;
use crate::errors::CompileError;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Unit,
    Array(Box<Type>),
    Maybe(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Struct(String),
    Enum(String),
    Unknown,
}

pub struct TypeChecker {
    filename: String,
    scopes: Vec<HashMap<String, Type>>,
    pub functions: HashMap<String, Type>,
    pub errors: Vec<CompileError>,
}

impl TypeChecker {
    pub fn new(filename: &str) -> Self {
        TypeChecker {
            filename: filename.to_string(),
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Function(func) => {
                    self.check_function(func);
                }
                TopLevel::Struct(s) => {
                    self.check_struct(s);
                }
                TopLevel::Enum(e) => {
                    self.check_enum(e);
                }
                TopLevel::Domain(d) => {
                    self.check_domain(d);
                }
            }
        }
    }

    fn check_domain(&mut self, domain: &crate::ast::DomainDef) {
        for item in &domain.items {
            match item {
                TopLevel::Function(func) => {
                    self.check_function(func);
                }
                TopLevel::Struct(s) => {
                    self.check_struct(s);
                }
                TopLevel::Enum(e) => {
                    self.check_enum(e);
                }
                TopLevel::Domain(d) => {
                    self.check_domain(d);
                }
            }
        }
    }

    fn check_function(&mut self, func: &FunctionDef) {
        let param_types: Vec<Type> = func.params.iter().map(|p| self.resolve_type(&p.ty)).collect();
        let return_type = func.return_type.as_ref().map(|t| self.resolve_type(t)).unwrap_or(Type::Unit);
        
        let func_type = Type::Function(param_types, Box::new(return_type.clone()));
        self.functions.insert(func.name.clone(), func_type);
        
        self.push_scope();
        
        for param in &func.params {
            let param_type = self.resolve_type(&param.ty);
            self.define(&param.name, param_type);
        }
        
        let block_type = self.check_block(&func.body);
        
        if block_type != return_type && return_type != Type::Unit {
            self.errors.push(CompileError::new(
                &format!("function {} return type mismatch: expected {:?}, got {:?}", func.name, return_type, block_type),
                &self.filename,
                0,
                0,
            ));
        }
        
        self.pop_scope();
    }

    fn check_struct(&mut self, s: &StructDef) {
        self.define(&s.name, Type::Struct(s.name.clone()));
    }

    fn check_enum(&mut self, e: &EnumDef) {
        self.define(&e.name, Type::Enum(e.name.clone()));
    }

    fn check_block(&mut self, block: &Block) -> Type {
        self.push_scope();
        
        let mut block_type = Type::Unit;
        
        for stmt in &block.stmts {
            block_type = self.check_stmt(stmt);
        }
        
        if let Some(result) = &block.result {
            block_type = self.check_expr(result);
        }
        
        self.pop_scope();
        
        block_type
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Type {
        match stmt {
            Stmt::Let { name, ty, value, mutable: _ } => {
                let value_type = self.check_expr(value);
                
                if let Some(declared_type) = ty {
                    let expected_type = self.resolve_type(declared_type);
                    if value_type != expected_type && value_type != Type::Unknown {
                        self.errors.push(CompileError::new(
                            &format!("type mismatch: expected {:?}, got {:?}", expected_type, value_type),
                            &self.filename,
                            0,
                            0,
                        ));
                    }
                }
                
                self.define(name, value_type);
                Type::Unit
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)
            }
            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.check_expr(expr)
                } else {
                    Type::Unit
                }
            }
            Stmt::Print(expr) => {
                self.check_expr(expr);
                Type::Unit
            }
            Stmt::While { condition, body } => {
                let cond_type = self.check_expr(condition);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.errors.push(CompileError::new(
                        &format!("while condition must be bool, got {:?}", cond_type),
                        &self.filename,
                        0,
                        0,
                    ));
                }
                self.check_block(body);
                Type::Unit
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Integer(_) => Type::Int,
            Expr::Float(_) => Type::Float,
            Expr::String(_) => Type::String,
            Expr::Bool(_) => Type::Bool,
            Expr::NoneValue => Type::Unknown,
            Expr::Identifier(name) => {
                self.lookup(name).unwrap_or(Type::Unknown)
            }
            Expr::BinaryOp { op, left, right } => {
                let left_type = self.check_expr(left);
                let right_type = self.check_expr(right);
                
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if left_type == Type::Int && right_type == Type::Int {
                            Type::Int
                        } else if left_type == Type::Float && right_type == Type::Float {
                            Type::Float
                        } else if left_type == Type::String && right_type == Type::String && *op == BinOp::Add {
                            Type::String
                        } else {
                            self.errors.push(CompileError::new(
                                &format!("invalid binary operation {:?} on {:?} and {:?}", op, left_type, right_type),
                                &self.filename,
                                0,
                                0,
                            ));
                            Type::Unknown
                        }
                    }
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => {
                        Type::Bool
                    }
                    BinOp::And | BinOp::Or => {
                        if left_type == Type::Bool && right_type == Type::Bool {
                            Type::Bool
                        } else {
                            self.errors.push(CompileError::new(
                                &format!("logical operation requires bool operands, got {:?} and {:?}", left_type, right_type),
                                &self.filename,
                                0,
                                0,
                            ));
                            Type::Unknown
                        }
                    }
                }
            }
            Expr::UnaryOp { op, expr } => {
                let expr_type = self.check_expr(expr);
                
                match op {
                    UnaryOp::Neg => {
                        if expr_type == Type::Int || expr_type == Type::Float {
                            expr_type
                        } else {
                            self.errors.push(CompileError::new(
                                &format!("negation requires int or float, got {:?}", expr_type),
                                &self.filename,
                                0,
                                0,
                            ));
                            Type::Unknown
                        }
                    }
                    UnaryOp::Not => {
                        if expr_type == Type::Bool {
                            Type::Bool
                        } else {
                            self.errors.push(CompileError::new(
                                &format!("logical not requires bool, got {:?}", expr_type),
                                &self.filename,
                                0,
                                0,
                            ));
                            Type::Unknown
                        }
                    }
                }
            }
            Expr::FunctionCall { name, args } => {
                if let Expr::Identifier(func_name) = name.as_ref() {
                    // Get function info first to avoid borrow issues
                    let func_info = self.functions.get(func_name).cloned();
                    
                    if let Some(Type::Function(param_types, return_type)) = func_info {
                        if args.len() != param_types.len() {
                            self.errors.push(CompileError::new(
                                &format!("{} expects {} arguments, got {}", func_name, param_types.len(), args.len()),
                                &self.filename,
                                0,
                                0,
                            ));
                            return Type::Unknown;
                        }
                        
                        for (arg, param_type) in args.iter().zip(param_types.iter()) {
                            let arg_type = self.check_expr(arg);
                            if arg_type != *param_type && arg_type != Type::Unknown {
                                self.errors.push(CompileError::new(
                                    &format!("argument type mismatch: expected {:?}, got {:?}", param_type, arg_type),
                                    &self.filename,
                                    0,
                                    0,
                                ));
                            }
                        }
                        
                        return *return_type;
                    }
                    
                    // Built-in functions
                    if func_name == "print" {
                        for arg in args {
                            self.check_expr(arg);
                        }
                        return Type::Unit;
                    }
                    
                    self.errors.push(CompileError::new(
                        &format!("undefined function: {}", func_name),
                        &self.filename,
                        0,
                        0,
                    ));
                    Type::Unknown
                } else {
                    Type::Unknown
                }
            }
            Expr::If { condition, then_body, else_body } => {
                let cond_type = self.check_expr(condition);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.errors.push(CompileError::new(
                        &format!("if condition must be bool, got {:?}", cond_type),
                        &self.filename,
                        0,
                        0,
                    ));
                }
                
                let then_type = self.check_block(then_body);
                
                if let Some(else_body) = else_body {
                    let else_type = self.check_block(else_body);
                    if then_type != else_type {
                        Type::Unknown
                    } else {
                        then_type
                    }
                } else {
                    Type::Unit
                }
            }
            Expr::While { condition, body } => {
                let cond_type = self.check_expr(condition);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.errors.push(CompileError::new(
                        &format!("while condition must be bool, got {:?}", cond_type),
                        &self.filename,
                        0,
                        0,
                    ));
                }
                self.check_block(body);
                Type::Unit
            }
            Expr::Block(block) => {
                self.check_block(block)
            }
            Expr::Assignment { name, value } => {
                if let Expr::Identifier(var_name) = name.as_ref() {
                    if let Some(var_type) = self.lookup(var_name) {
                        let value_type = self.check_expr(value);
                        if var_type != value_type && value_type != Type::Unknown {
                            self.errors.push(CompileError::new(
                                &format!("type mismatch in assignment: expected {:?}, got {:?}", var_type, value_type),
                                &self.filename,
                                0,
                                0,
                            ));
                        }
                        var_type
                    } else {
                        self.errors.push(CompileError::new(
                            &format!("undefined variable: {}", var_name),
                            &self.filename,
                            0,
                            0,
                        ));
                        Type::Unknown
                    }
                } else {
                    Type::Unknown
                }
            }
            Expr::FieldAccess { object, field: _ } => {
                self.check_expr(object);
                Type::Unknown
            }
            Expr::MethodCall { object, method: _, args } => {
                self.check_expr(object);
                for arg in args {
                    self.check_expr(arg);
                }
                Type::Unknown
            }
            Expr::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    Type::Array(Box::new(Type::Unknown))
                } else {
                    let elem_type = self.check_expr(&elements[0]);
                    Type::Array(Box::new(elem_type))
                }
            }
            Expr::ArrayAccess { array, index } => {
                let array_type = self.check_expr(array);
                let index_type = self.check_expr(index);
                
                if index_type != Type::Int && index_type != Type::Unknown {
                    self.errors.push(CompileError::new(
                        &format!("array index must be int, got {:?}", index_type),
                        &self.filename,
                        0,
                        0,
                    ));
                }
                
                if let Type::Array(elem_type) = array_type {
                    *elem_type
                } else {
                    Type::Unknown
                }
            }
            Expr::TupleLiteral(elements) => {
                Type::Unknown
            }
            Expr::SomeValue(value) => {
                let value_type = self.check_expr(value);
                Type::Maybe(Box::new(value_type))
            }
            Expr::StructLiteral { name, fields } => {
                for (_, value) in fields {
                    self.check_expr(value);
                }
                Type::Struct(name.clone())
            }
            Expr::QualifiedName { module: _, name: _ } => {
                Type::Unknown
            }
        }
    }

    fn resolve_type(&self, ann: &TypeAnnotation) -> Type {
        match ann {
            TypeAnnotation::Simple(name) => {
                match name.as_str() {
                    "Int" | "i64" => Type::Int,
                    "Float" | "f64" => Type::Float,
                    "String" => Type::String,
                    "Bool" => Type::Bool,
                    "()" => Type::Unit,
                    _ => Type::Struct(name.clone()),
                }
            }
            TypeAnnotation::Generic(name, args) => {
                if name == "List" || name == "Array" {
                    if let Some(arg) = args.first() {
                        Type::Array(Box::new(self.resolve_type(arg)))
                    } else {
                        Type::Array(Box::new(Type::Unknown))
                    }
                } else {
                    Type::Unknown
                }
            }
            TypeAnnotation::Maybe(inner) => {
                Type::Maybe(Box::new(self.resolve_type(inner)))
            }
            TypeAnnotation::Reference(inner) => {
                self.resolve_type(inner)
            }
            TypeAnnotation::MutableReference(inner) => {
                self.resolve_type(inner)
            }
            TypeAnnotation::Tuple(_) => Type::Unknown,
            TypeAnnotation::Function(_, _) => Type::Unknown,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }
}
