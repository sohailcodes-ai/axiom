#[derive(Debug, Clone)]
pub enum HIRExpr {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
    LoadLocal(usize),
    StoreLocal(usize, Box<HIRStmt>),
    BinaryOp {
        op: HIRBinOp,
        left: Box<HIRExpr>,
        right: Box<HIRExpr>,
    },
    UnaryOp {
        op: HIRUnaryOp,
        expr: Box<HIRExpr>,
    },
    Call {
        func: String,
        args: Vec<HIRExpr>,
    },
    CrossDomainCall {
        func: String,
        args: Vec<HIRExpr>,
        from_domain: String,
        to_domain: String,
    },
    If {
        condition: Box<HIRExpr>,
        then_body: Vec<HIRStmt>,
        else_body: Option<Vec<HIRStmt>>,
    },
    While {
        condition: Box<HIRExpr>,
        body: Vec<HIRStmt>,
    },
    Print(Box<HIRExpr>),
    Return(Box<HIRExpr>),
    MakeStruct {
        fields: Vec<HIRExpr>,
    },
    GetField {
        object: Box<HIRExpr>,
        index: usize,
    },
    MakeArray {
        elements: Vec<HIRExpr>,
    },
    ArrayGet {
        array: Box<HIRExpr>,
        index: Box<HIRExpr>,
    },
}

#[derive(Debug, Clone)]
pub enum HIRBinOp {
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
pub enum HIRUnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum HIRStmt {
    Let(usize, HIRExpr),
    Assign(usize, HIRExpr),
    Expr(HIRExpr),
    Return(Option<HIRExpr>),
    Print(HIRExpr),
    While {
        condition: HIRExpr,
        body: Vec<HIRStmt>,
    },
    If {
        condition: HIRExpr,
        then_body: Vec<HIRStmt>,
        else_body: Option<Vec<HIRStmt>>,
    },
}

#[derive(Debug, Clone)]
pub struct HIRFunction {
    pub name: String,
    pub params: Vec<String>,
    pub param_count: usize,
    pub locals_count: usize,
    pub body: Vec<HIRStmt>,
    pub is_main: bool,
}

#[derive(Debug, Clone)]
pub struct HIRProgram {
    pub functions: Vec<HIRFunction>,
}
