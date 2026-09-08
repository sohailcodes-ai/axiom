#[derive(Debug, Clone)]
pub enum TypeAnnotation {
    Simple(String),
    Generic(String, Vec<TypeAnnotation>),
    Maybe(Box<TypeAnnotation>),
    Reference(Box<TypeAnnotation>),
    MutableReference(Box<TypeAnnotation>),
    Tuple(Vec<TypeAnnotation>),
    Function(Vec<TypeAnnotation>, Box<TypeAnnotation>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Identifier(String),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    FunctionCall {
        name: Box<Expr>,
        args: Vec<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_body: Block,
        else_body: Option<Block>,
    },
    While {
        condition: Box<Expr>,
        body: Block,
    },
    Block(Block),
    Assignment {
        name: Box<Expr>,
        value: Box<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    ArrayLiteral(Vec<Expr>),
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    TupleLiteral(Vec<Expr>),
    SomeValue(Box<Expr>),
    NoneValue,
    StructLiteral {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    QualifiedName {
        module: String,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub result: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<TypeAnnotation>,
        value: Expr,
        mutable: bool,
    },
    Expr(Expr),
    Return(Option<Expr>),
    Print(Expr),
    While {
        condition: Expr,
        body: Block,
    },
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeAnnotation,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, TypeAnnotation)>,
}

#[derive(Debug, Clone)]
pub enum VariantDef {
    Unit(String),
    Tuple(String, Vec<TypeAnnotation>),
    Struct(String, Vec<(String, TypeAnnotation)>),
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<VariantDef>,
}

#[derive(Debug, Clone)]
pub struct DomainDef {
    pub name: String,
    pub items: Vec<TopLevel>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Function(FunctionDef),
    Struct(StructDef),
    Enum(EnumDef),
    Domain(DomainDef),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<TopLevel>,
}
