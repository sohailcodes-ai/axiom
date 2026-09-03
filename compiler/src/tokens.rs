use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    
    // Identifiers
    Identifier(String),
    
    // Keywords
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    Return,
    Print,
    True,
    False,
    Struct,
    Enum,
    Domain,
    Spawn,
    Await,
    Chan,
    New,
    Some,
    None,
    Maybe,
    Type,
    As,
    In,
    Loop,
    Break,
    Continue,
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    Not,
    Assign,
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Arrow,
    DoubleColon,
    Dot,
    Question,
    
    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Token { kind, line, column }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Integer(n) => write!(f, "{}", n),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::Bool(b) => write!(f, "{}", b),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Print => write!(f, "print"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Domain => write!(f, "domain"),
            TokenKind::Spawn => write!(f, "spawn"),
            TokenKind::Await => write!(f, "await"),
            TokenKind::Chan => write!(f, "chan"),
            TokenKind::New => write!(f, "new"),
            TokenKind::Some => write!(f, "Some"),
            TokenKind::None => write!(f, "None"),
            TokenKind::Maybe => write!(f, "maybe"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::As => write!(f, "as"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Eq => write!(f, "=="),
            TokenKind::Neq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Lte => write!(f, "<="),
            TokenKind::Gte => write!(f, ">="),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Assign => write!(f, "="),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::DoubleColon => write!(f, "::"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Question => write!(f, "?"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}
