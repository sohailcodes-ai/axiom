use crate::ast::*;
use crate::errors::CompileError;
use crate::tokens::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    filename: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, filename: &str) -> Self {
        Parser {
            tokens,
            pos: 0,
            filename: filename.to_string(),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, CompileError> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            items.push(self.parse_top_level()?);
        }
        
        Ok(Program { items })
    }

    fn parse_top_level(&mut self) -> Result<TopLevel, CompileError> {
        if self.check(&TokenKind::Fn) {
            Ok(TopLevel::Function(self.parse_function()?))
        } else if self.check(&TokenKind::Struct) {
            Ok(TopLevel::Struct(self.parse_struct()?))
        } else if self.check(&TokenKind::Enum) {
            Ok(TopLevel::Enum(self.parse_enum()?))
        } else {
            Err(self.error("expected top-level declaration"))
        }
    }

    fn parse_function(&mut self) -> Result<FunctionDef, CompileError> {
        self.consume(&TokenKind::Fn, "expected 'fn'")?;
        let name = self.consume_identifier("expected function name")?;
        
        self.consume(&TokenKind::LParen, "expected '('")?;
        let mut params = Vec::new();
        
        if !self.check(&TokenKind::RParen) {
            loop {
                let param_name = self.consume_identifier("expected parameter name")?;
                self.consume(&TokenKind::Colon, "expected ':'")?;
                let param_type = self.parse_type()?;
                params.push(Param {
                    name: param_name,
                    ty: param_type,
                });
                
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        self.consume(&TokenKind::RParen, "expected ')'")?;
        
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        Ok(FunctionDef {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_struct(&mut self) -> Result<StructDef, CompileError> {
        self.consume(&TokenKind::Struct, "expected 'struct'")?;
        let name = self.consume_identifier("expected struct name")?;
        
        self.consume(&TokenKind::LBrace, "expected '{'")?;
        let mut fields = Vec::new();
        
        while !self.check(&TokenKind::RBrace) {
            let field_name = self.consume_identifier("expected field name")?;
            self.consume(&TokenKind::Colon, "expected ':'")?;
            let field_type = self.parse_type()?;
            fields.push((field_name, field_type));
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        self.consume(&TokenKind::RBrace, "expected '}'")?;
        
        Ok(StructDef { name, fields })
    }

    fn parse_enum(&mut self) -> Result<EnumDef, CompileError> {
        self.consume(&TokenKind::Enum, "expected 'enum'")?;
        let name = self.consume_identifier("expected enum name")?;
        
        self.consume(&TokenKind::LBrace, "expected '{'")?;
        let mut variants = Vec::new();
        
        while !self.check(&TokenKind::RBrace) {
            let variant_name = self.consume_identifier("expected variant name")?;
            
            let variant = if self.check(&TokenKind::LParen) {
                self.advance();
                let mut types = Vec::new();
                
                if !self.check(&TokenKind::RParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                
                self.consume(&TokenKind::RParen, "expected ')'")?;
                VariantDef::Tuple(variant_name, types)
            } else if self.check(&TokenKind::LBrace) {
                self.advance();
                let mut fields = Vec::new();
                
                while !self.check(&TokenKind::RBrace) {
                    let field_name = self.consume_identifier("expected field name")?;
                    self.consume(&TokenKind::Colon, "expected ':'")?;
                    let field_type = self.parse_type()?;
                    fields.push((field_name, field_type));
                    
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                
                self.consume(&TokenKind::RBrace, "expected '}'")?;
                VariantDef::Struct(variant_name, fields)
            } else {
                VariantDef::Unit(variant_name)
            };
            
            variants.push(variant);
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        
        self.consume(&TokenKind::RBrace, "expected '}'")?;
        
        Ok(EnumDef { name, variants })
    }

    fn parse_type(&mut self) -> Result<TypeAnnotation, CompileError> {
        if self.check(&TokenKind::Maybe) {
            self.advance();
            let inner = self.parse_type()?;
            return Ok(TypeAnnotation::Maybe(Box::new(inner)));
        }
        
        let name = self.consume_identifier("expected type name")?;
        
        if self.check(&TokenKind::LBracket) {
            self.advance();
            let mut args = Vec::new();
            
            if !self.check(&TokenKind::RBracket) {
                loop {
                    args.push(self.parse_type()?);
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
            }
            
            self.consume(&TokenKind::RBracket, "expected ']'")?;
            return Ok(TypeAnnotation::Generic(name, args));
        }
        
        Ok(TypeAnnotation::Simple(name))
    }

    fn parse_block(&mut self) -> Result<Block, CompileError> {
        self.consume(&TokenKind::LBrace, "expected '{'")?;
        let mut stmts = Vec::new();
        let mut result = None;
        
        while !self.check(&TokenKind::RBrace) {
            if self.check(&TokenKind::Let) {
                stmts.push(self.parse_let_stmt()?);
            } else if self.check(&TokenKind::Return) {
                stmts.push(self.parse_return_stmt()?);
            } else if self.check(&TokenKind::Print) {
                stmts.push(self.parse_print_stmt()?);
            } else if self.check(&TokenKind::While) {
                stmts.push(self.parse_while_stmt()?);
            } else if self.check(&TokenKind::If) {
                let expr = self.parse_if_expr()?;
                stmts.push(Stmt::Expr(expr));
            } else {
                let expr = self.parse_expr()?;
                stmts.push(Stmt::Expr(expr));
            }
        }
        
        self.consume(&TokenKind::RBrace, "expected '}'")?;
        
        Ok(Block { stmts, result })
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, CompileError> {
        self.consume(&TokenKind::Let, "expected 'let'")?;
        
        let mutable = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };
        
        let name = self.consume_identifier("expected variable name")?;
        
        let ty = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.consume(&TokenKind::Assign, "expected '='")?;
        let value = self.parse_expr()?;
        self.consume(&TokenKind::Semicolon, "expected ';'")?;
        
        Ok(Stmt::Let {
            name,
            ty,
            value,
            mutable,
        })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, CompileError> {
        self.consume(&TokenKind::Return, "expected 'return'")?;
        
        let value = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        
        self.consume(&TokenKind::Semicolon, "expected ';'")?;
        
        Ok(Stmt::Return(value))
    }

    fn parse_print_stmt(&mut self) -> Result<Stmt, CompileError> {
        self.consume(&TokenKind::Print, "expected 'print'")?;
        self.consume(&TokenKind::LParen, "expected '('")?;
        let expr = self.parse_expr()?;
        self.consume(&TokenKind::RParen, "expected ')'")?;
        self.consume(&TokenKind::Semicolon, "expected ';'")?;
        
        Ok(Stmt::Print(expr))
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, CompileError> {
        self.consume(&TokenKind::While, "expected 'while'")?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        
        Ok(Stmt::While { condition, body })
    }

    fn parse_if_expr(&mut self) -> Result<Expr, CompileError> {
        self.consume(&TokenKind::If, "expected 'if'")?;
        let condition = self.parse_expr()?;
        let then_body = self.parse_block()?;
        
        let else_body = if self.check(&TokenKind::Else) {
            self.advance();
            if self.check(&TokenKind::If) {
                let else_if = self.parse_if_expr()?;
                Some(Block {
                    stmts: vec![Stmt::Expr(else_if)],
                    result: None,
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        
        Ok(Expr::If {
            condition: Box::new(condition),
            then_body,
            else_body,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, CompileError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_and()?;
        
        while self.check(&TokenKind::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_equality()?;
        
        while self.check(&TokenKind::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_comparison()?;
        
        loop {
            if self.check(&TokenKind::Eq) {
                self.advance();
                let right = self.parse_comparison()?;
                left = Expr::BinaryOp {
                    op: BinOp::Eq,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&TokenKind::Neq) {
                self.advance();
                let right = self.parse_comparison()?;
                left = Expr::BinaryOp {
                    op: BinOp::Neq,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_addition()?;
        
        loop {
            if self.check(&TokenKind::Lt) {
                self.advance();
                let right = self.parse_addition()?;
                left = Expr::BinaryOp {
                    op: BinOp::Lt,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&TokenKind::Gt) {
                self.advance();
                let right = self.parse_addition()?;
                left = Expr::BinaryOp {
                    op: BinOp::Gt,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&TokenKind::Lte) {
                self.advance();
                let right = self.parse_addition()?;
                left = Expr::BinaryOp {
                    op: BinOp::Lte,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&TokenKind::Gte) {
                self.advance();
                let right = self.parse_addition()?;
                left = Expr::BinaryOp {
                    op: BinOp::Gte,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_multiplication()?;
        
        loop {
            if self.check(&TokenKind::Plus) {
                self.advance();
                let right = self.parse_multiplication()?;
                left = Expr::BinaryOp {
                    op: BinOp::Add,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&TokenKind::Minus) {
                self.advance();
                let right = self.parse_multiplication()?;
                left = Expr::BinaryOp {
                    op: BinOp::Sub,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_unary()?;
        
        loop {
            if self.check(&TokenKind::Star) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::BinaryOp {
                    op: BinOp::Mul,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&TokenKind::Slash) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::BinaryOp {
                    op: BinOp::Div,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&TokenKind::Percent) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::BinaryOp {
                    op: BinOp::Mod,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, CompileError> {
        if self.check(&TokenKind::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::UnaryOp {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            })
        } else if self.check(&TokenKind::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::UnaryOp {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            })
        } else {
            self.parse_call()
        }
    }

    fn parse_call(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.check(&TokenKind::LParen) {
                self.advance();
                let mut args = Vec::new();
                
                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                
                self.consume(&TokenKind::RParen, "expected ')'")?;
                expr = Expr::FunctionCall {
                    name: Box::new(expr),
                    args,
                };
            } else if self.check(&TokenKind::Dot) {
                self.advance();
                let field = self.consume_identifier("expected field or method name")?;
                
                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if !self.check(&TokenKind::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }
                    
                    self.consume(&TokenKind::RParen, "expected ')'")?;
                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        method: field,
                        args,
                    };
                } else {
                    expr = Expr::FieldAccess {
                        object: Box::new(expr),
                        field,
                    };
                }
            } else if self.check(&TokenKind::LBracket) {
                self.advance();
                let index = self.parse_expr()?;
                self.consume(&TokenKind::RBracket, "expected ']'")?;
                expr = Expr::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.check(&TokenKind::DoubleColon) {
                self.advance();
                let name = self.consume_identifier("expected name")?;
                
                if let Expr::Identifier(module) = expr {
                    expr = Expr::QualifiedName { module, name };
                } else {
                    return Err(self.error("expected module name before '::'"));
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, CompileError> {
        match &self.peek().kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                return Ok(Expr::Integer(n));
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                return Ok(Expr::Float(n));
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                return Ok(Expr::String(s));
            }
            TokenKind::Bool(b) => {
                let b = *b;
                self.advance();
                return Ok(Expr::Bool(b));
            }
            TokenKind::None => {
                self.advance();
                return Ok(Expr::NoneValue);
            }
            TokenKind::Some => {
                self.advance();
                self.consume(&TokenKind::LParen, "expected '('")?;
                let value = self.parse_expr()?;
                self.consume(&TokenKind::RParen, "expected ')'")?;
                return Ok(Expr::SomeValue(Box::new(value)));
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut fields = Vec::new();
                    
                    while !self.check(&TokenKind::RBrace) {
                        let field_name = self.consume_identifier("expected field name")?;
                        self.consume(&TokenKind::Colon, "expected ':'")?;
                        let value = self.parse_expr()?;
                        fields.push((field_name, value));
                        
                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                    }
                    
                    self.consume(&TokenKind::RBrace, "expected '}'")?;
                    return Ok(Expr::StructLiteral { name, fields });
                }
                
                return Ok(Expr::Identifier(name));
            }
            _ => {}
        }
        
        if self.check(&TokenKind::LParen) {
            self.advance();
            
            if self.check(&TokenKind::RParen) {
                self.advance();
                return Ok(Expr::TupleLiteral(Vec::new()));
            }
            
            let first = self.parse_expr()?;
            
            if self.check(&TokenKind::Comma) {
                self.advance();
                let mut elements = vec![first];
                
                while !self.check(&TokenKind::RParen) {
                    elements.push(self.parse_expr()?);
                    if !self.check(&TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                
                self.consume(&TokenKind::RParen, "expected ')'")?;
                return Ok(Expr::TupleLiteral(elements));
            }
            
            self.consume(&TokenKind::RParen, "expected ')'")?;
            return Ok(first);
        }
        
        if self.check(&TokenKind::LBracket) {
            self.advance();
            let mut elements = Vec::new();
            
            while !self.check(&TokenKind::RBracket) {
                elements.push(self.parse_expr()?);
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            
            self.consume(&TokenKind::RBracket, "expected ']'")?;
            return Ok(Expr::ArrayLiteral(elements));
        }
        
        if self.check(&TokenKind::If) {
            return self.parse_if_expr();
        }
        
        if self.check(&TokenKind::LBrace) {
            let block = self.parse_block()?;
            return Ok(Expr::Block(block));
        }
        
        Err(self.error("unexpected token"))
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.pos >= self.tokens.len() {
            return false;
        }
        std::mem::discriminant(&self.tokens[self.pos].kind) == std::mem::discriminant(kind)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::Eof
    }

    fn consume(&mut self, kind: &TokenKind, message: &str) -> Result<&Token, CompileError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, CompileError> {
        if let TokenKind::Identifier(name) = &self.peek().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.error(message))
        }
    }

    fn error(&self, message: &str) -> CompileError {
        if self.pos < self.tokens.len() {
            CompileError::new(
                message,
                &self.filename,
                self.tokens[self.pos].line,
                self.tokens[self.pos].column,
            )
        } else {
            CompileError::new(message, &self.filename, 0, 0)
        }
    }
}
