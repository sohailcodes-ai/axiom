use std::collections::HashMap;
use crate::ast::*;
use crate::errors::CompileError;
use crate::type_checker::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainKind {
    Server,
    Client,
}

impl std::fmt::Display for DomainKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainKind::Server => write!(f, "server"),
            DomainKind::Client => write!(f, "client"),
        }
    }
}

pub struct DomainAnalyzer {
    filename: String,
    function_domains: HashMap<String, DomainKind>,
    functions: HashMap<String, Type>,
    struct_fields: HashMap<String, Vec<(String, Type)>>,
    type_domains: HashMap<String, DomainKind>,
    has_domains: bool,
    pub errors: Vec<CompileError>,
}

impl DomainAnalyzer {
    pub fn new(filename: &str, functions: HashMap<String, Type>) -> Self {
        DomainAnalyzer {
            filename: filename.to_string(),
            function_domains: HashMap::new(),
            functions,
            struct_fields: HashMap::new(),
            type_domains: HashMap::new(),
            has_domains: false,
            errors: Vec::new(),
        }
    }

    pub fn analyze_program(&mut self, program: &Program) {
        self.collect_domains(program);
        self.collect_structs(program);
        self.collect_type_domains(program);
        for item in &program.items {
            self.analyze_item(item);
        }
        self.check_domain_enforcement(program);
        self.check_cross_domain_calls(program);
    }

    fn collect_domains(&mut self, program: &Program) {
        for item in &program.items {
            if let TopLevel::Domain(_) = item {
                self.has_domains = true;
                return;
            }
        }
    }

    fn collect_type_domains(&mut self, program: &Program) {
        for item in &program.items {
            self.collect_type_domains_item(item, None);
        }
    }

    fn collect_type_domains_item(&mut self, item: &TopLevel, current_domain: Option<DomainKind>) {
        match item {
            TopLevel::Struct(s) => {
                if let Some(domain) = current_domain {
                    self.type_domains.insert(s.name.clone(), domain);
                }
            }
            TopLevel::Domain(d) => {
                let kind = match d.name.as_str() {
                    "server" => DomainKind::Server,
                    "client" => DomainKind::Client,
                    _ => return,
                };
                for inner in &d.items {
                    self.collect_type_domains_item(inner, Some(kind));
                }
            }
            _ => {}
        }
    }

    fn check_domain_enforcement(&mut self, program: &Program) {
        if !self.has_domains {
            return;
        }

        for item in &program.items {
            self.check_item_domain_enforcement(item);
        }
    }

    fn check_item_domain_enforcement(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function(f) => {
                if !self.function_domains.contains_key(&f.name) {
                    self.errors.push(CompileError::new(
                        &format!(
                            "function '{}' must be declared inside a domain block when domains are used in the program",
                            f.name
                        ),
                        &self.filename,
                        0,
                        0,
                    ));
                }
            }
            TopLevel::Domain(d) => {
                for inner in &d.items {
                    self.check_item_domain_enforcement(inner);
                }
            }
            _ => {}
        }
    }

    fn collect_structs(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Struct(s) => {
                    let fields: Vec<(String, Type)> = s.fields.iter()
                        .map(|(name, ann)| (name.clone(), self.resolve_type_ann(ann)))
                        .collect();
                    self.struct_fields.insert(s.name.clone(), fields);
                }
                TopLevel::Domain(d) => self.collect_structs_from_domain(d),
                _ => {}
            }
        }
    }

    fn collect_structs_from_domain(&mut self, domain: &DomainDef) {
        for item in &domain.items {
            match item {
                TopLevel::Struct(s) => {
                    let fields: Vec<(String, Type)> = s.fields.iter()
                        .map(|(name, ann)| (name.clone(), self.resolve_type_ann(ann)))
                        .collect();
                    self.struct_fields.insert(s.name.clone(), fields);
                }
                TopLevel::Domain(d) => self.collect_structs_from_domain(d),
                _ => {}
            }
        }
    }

    fn resolve_type_ann(&self, ann: &crate::ast::TypeAnnotation) -> Type {
        match ann {
            crate::ast::TypeAnnotation::Simple(name) => {
                match name.as_str() {
                    "Int" | "i64" => Type::Int,
                    "Float" | "f64" => Type::Float,
                    "String" => Type::String,
                    "Bool" => Type::Bool,
                    "()" => Type::Unit,
                    _ => Type::Struct(name.clone()),
                }
            }
            crate::ast::TypeAnnotation::Generic(name, args) => {
                if name == "List" || name == "Array" {
                    if let Some(arg) = args.first() {
                        Type::Array(Box::new(self.resolve_type_ann(arg)))
                    } else {
                        Type::Array(Box::new(Type::Unknown))
                    }
                } else {
                    Type::Unknown
                }
            }
            crate::ast::TypeAnnotation::Maybe(inner) => {
                Type::Maybe(Box::new(self.resolve_type_ann(inner)))
            }
            _ => Type::Unknown,
        }
    }

    fn analyze_item(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Domain(d) => {
                self.analyze_domain(d);
            }
            TopLevel::Function(f) => {
                self.analyze_function(f, None);
            }
            _ => {}
        }
    }

    fn analyze_domain(&mut self, domain: &DomainDef) {
        let kind = match domain.name.as_str() {
            "server" => DomainKind::Server,
            "client" => DomainKind::Client,
            other => {
                self.errors.push(CompileError::new(
                    &format!("unknown domain: '{}'. AXIOM supports 'server' and 'client'", other),
                    &self.filename,
                    0,
                    0,
                ));
                return;
            }
        };

        for item in &domain.items {
            match item {
                TopLevel::Function(f) => {
                    self.analyze_function(f, Some(kind));
                }
                TopLevel::Domain(d) => {
                    self.analyze_domain(d);
                }
                _ => {}
            }
        }
    }

    fn analyze_function(&mut self, func: &FunctionDef, domain: Option<DomainKind>) {
        if let Some(kind) = domain {
            self.function_domains.insert(func.name.clone(), kind);
        }
    }

    fn check_cross_domain_calls(&mut self, program: &Program) {
        for item in program.items.iter() {
            self.check_item_calls(item);
        }
    }

    fn check_item_calls(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function(f) => {
                self.check_function_calls(f);
            }
            TopLevel::Domain(d) => {
                for inner in &d.items {
                    self.check_item_calls(inner);
                }
            }
            _ => {}
        }
    }

    fn check_function_calls(&mut self, func: &FunctionDef) {
        let caller_domain = self.function_domains.get(&func.name).copied();
        self.check_block_calls(&func.body, caller_domain, &func.name);
    }

    fn check_block_calls(&mut self, block: &Block, caller_domain: Option<DomainKind>, caller_name: &str) {
        for stmt in &block.stmts {
            self.check_stmt_calls(stmt, caller_domain, caller_name);
        }
    }

    fn check_stmt_calls(&mut self, stmt: &Stmt, caller_domain: Option<DomainKind>, caller_name: &str) {
        match stmt {
            Stmt::Let { value, .. } => {
                self.check_expr_calls(value, caller_domain, caller_name);
            }
            Stmt::Expr(expr) => {
                self.check_expr_calls(expr, caller_domain, caller_name);
            }
            Stmt::Return(Some(expr)) => {
                self.check_expr_calls(expr, caller_domain, caller_name);
            }
            Stmt::Print(expr) => {
                self.check_expr_calls(expr, caller_domain, caller_name);
            }
            Stmt::While { condition, body } => {
                self.check_expr_calls(condition, caller_domain, caller_name);
                self.check_block_calls(body, caller_domain, caller_name);
            }
            _ => {}
        }
    }

    fn check_expr_calls(&mut self, expr: &Expr, caller_domain: Option<DomainKind>, caller_name: &str) {
        match expr {
            Expr::FunctionCall { name, args } => {
                if let Expr::Identifier(func_name) = name.as_ref() {
                    for arg in args {
                        self.check_expr_calls(arg, caller_domain, caller_name);
                    }

                    if let (Some(caller_d), Some(callee_d)) = (
                        caller_domain,
                        self.function_domains.get(func_name).copied(),
                    ) {
                        if caller_d != callee_d {
                            self.check_cross_domain_validity(
                                caller_name,
                                caller_d,
                                func_name,
                                callee_d,
                                args,
                            );
                        }
                    }
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                self.check_expr_calls(left, caller_domain, caller_name);
                self.check_expr_calls(right, caller_domain, caller_name);
            }
            Expr::UnaryOp { expr, .. } => {
                self.check_expr_calls(expr, caller_domain, caller_name);
            }
            Expr::If { condition, then_body, else_body } => {
                self.check_expr_calls(condition, caller_domain, caller_name);
                self.check_block_calls(then_body, caller_domain, caller_name);
                if let Some(else_b) = else_body {
                    self.check_block_calls(else_b, caller_domain, caller_name);
                }
            }
            Expr::While { condition, body } => {
                self.check_expr_calls(condition, caller_domain, caller_name);
                self.check_block_calls(body, caller_domain, caller_name);
            }
            Expr::Block(block) => {
                self.check_block_calls(block, caller_domain, caller_name);
            }
            Expr::Assignment { name, value } => {
                self.check_expr_calls(name, caller_domain, caller_name);
                self.check_expr_calls(value, caller_domain, caller_name);
            }
            Expr::MethodCall { object, args, .. } => {
                self.check_expr_calls(object, caller_domain, caller_name);
                for arg in args {
                    self.check_expr_calls(arg, caller_domain, caller_name);
                }
            }
            Expr::ArrayLiteral(elements) => {
                for e in elements {
                    self.check_expr_calls(e, caller_domain, caller_name);
                }
            }
            Expr::ArrayAccess { array, index } => {
                self.check_expr_calls(array, caller_domain, caller_name);
                self.check_expr_calls(index, caller_domain, caller_name);
            }
            Expr::SomeValue(value) => {
                self.check_expr_calls(value, caller_domain, caller_name);
            }
            Expr::StructLiteral { fields, .. } => {
                for (_, v) in fields {
                    self.check_expr_calls(v, caller_domain, caller_name);
                }
            }
            Expr::TupleLiteral(elements) => {
                for e in elements {
                    self.check_expr_calls(e, caller_domain, caller_name);
                }
            }
            _ => {}
        }
    }

    fn check_cross_domain_validity(
        &mut self,
        caller_name: &str,
        caller_domain: DomainKind,
        callee_name: &str,
        callee_domain: DomainKind,
        args: &[Expr],
    ) {
        if caller_domain == DomainKind::Client && callee_domain == DomainKind::Server {
            let callee_type = self.functions.get(callee_name);
            if let Some(Type::Function(param_types, return_type)) = callee_type {
                for (i, arg) in args.iter().enumerate() {
                    let arg_type = self.infer_expr_type(arg);
                    if let Some(at) = arg_type {
                        if i < param_types.len() && !self.is_transmissible(&param_types[i]) {
                            self.errors.push(CompileError::new(
                                &format!(
                                    "cross-domain argument {} of call to '{}' is not transmissible (type: {:?})",
                                    i + 1, callee_name, param_types[i]
                                ),
                                &self.filename,
                                0,
                                0,
                            ));
                        }
                        let _ = at;
                    }
                }
                if !self.is_transmissible(return_type) {
                    self.errors.push(CompileError::new(
                        &format!(
                            "return type of '{}' is not transmissible (type: {:?}), cannot cross from {} to {}",
                            callee_name, return_type, caller_domain, callee_domain
                        ),
                        &self.filename,
                        0,
                        0,
                    ));
                }
                // Check for domain-local types crossing boundaries
                if let Type::Struct(name) = return_type.as_ref() {
                    if let Some(type_domain) = self.type_domains.get(name) {
                        if *type_domain != caller_domain {
                            self.errors.push(CompileError::new(
                                &format!(
                                    "type '{}' is local to {} domain and cannot cross to {} domain",
                                    name, type_domain, caller_domain
                                ),
                                &self.filename,
                                0,
                                0,
                            ));
                        }
                    }
                }
            }
        } else if caller_domain == DomainKind::Server && callee_domain == DomainKind::Client {
            self.errors.push(CompileError::new(
                &format!(
                    "server->client calls are not supported: server function '{}' cannot call client function '{}'. Use events/messages for server-to-client communication",
                    caller_name, callee_name
                ),
                &self.filename,
                0,
                0,
            ));
        }
    }

    fn infer_expr_type(&self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Integer(_) => Some(Type::Int),
            Expr::Float(_) => Some(Type::Float),
            Expr::String(_) => Some(Type::String),
            Expr::Bool(_) => Some(Type::Bool),
            Expr::NoneValue => Some(Type::Unknown),
            Expr::Identifier(name) => {
                self.functions.get(name).cloned().or(Some(Type::Unknown))
            }
            Expr::FunctionCall { name, .. } => {
                if let Expr::Identifier(func_name) = name.as_ref() {
                    if let Some(Type::Function(_, ret)) = self.functions.get(func_name) {
                        Some(*ret.clone())
                    } else {
                        Some(Type::Unknown)
                    }
                } else {
                    Some(Type::Unknown)
                }
            }
            Expr::BinaryOp { .. } => Some(Type::Unknown),
            Expr::UnaryOp { .. } => Some(Type::Unknown),
            Expr::StructLiteral { name, .. } => Some(Type::Struct(name.clone())),
            _ => Some(Type::Unknown),
        }
    }

    pub fn is_transmissible(&self, ty: &Type) -> bool {
        match ty {
            Type::Int | Type::Float | Type::String | Type::Bool => true,
            Type::Unit => true,
            Type::Array(inner) => self.is_transmissible(inner),
            Type::Maybe(inner) => self.is_transmissible(inner),
            Type::Struct(name) => self.is_struct_transmissible(name),
            Type::Enum(name) => self.is_enum_transmissible(name),
            Type::Function(_, _) => false,
            Type::Unknown => false,
        }
    }

    fn is_struct_transmissible(&self, name: &str) -> bool {
        match self.struct_fields.get(name) {
            Some(fields) => {
                fields.iter().all(|(_, ty)| self.is_transmissible(ty))
            }
            None => true,
        }
    }

    fn is_enum_transmissible(&self, _name: &str) -> bool {
        true
    }

    pub fn get_function_domain(&self, name: &str) -> Option<DomainKind> {
        self.function_domains.get(name).copied()
    }

    pub fn get_all_function_domains(&self) -> HashMap<String, DomainKind> {
        self.function_domains.clone()
    }

    pub fn get_struct_fields(&self) -> &HashMap<String, Vec<(String, Type)>> {
        &self.struct_fields
    }

    pub fn has_domains(&self) -> bool {
        self.has_domains
    }
}
