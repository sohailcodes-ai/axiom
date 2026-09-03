use crate::tokens::{Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    filename: String,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str, filename: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            filename: filename.to_string(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        
        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        
        if self.pos >= self.source.len() {
            return Token::new(TokenKind::Eof, self.line, self.column);
        }

        let ch = self.source[self.pos];
        let start_line = self.line;
        let start_col = self.column;

        // String literals
        if ch == '"' {
            return self.read_string(start_line, start_col);
        }

        // Numbers
        if ch.is_ascii_digit() {
            return self.read_number(start_line, start_col);
        }

        // Identifiers and keywords
        if ch.is_ascii_alphabetic() || ch == '_' {
            return self.read_identifier(start_line, start_col);
        }

        // Two-character tokens
        if self.pos + 1 < self.source.len() {
            let next = self.source[self.pos + 1];
            let two_char = format!("{}{}", ch, next);
            
            match two_char.as_str() {
                "==" => { self.advance(2); return Token::new(TokenKind::Eq, start_line, start_col); }
                "!=" => { self.advance(2); return Token::new(TokenKind::Neq, start_line, start_col); }
                "<=" => { self.advance(2); return Token::new(TokenKind::Lte, start_line, start_col); }
                ">=" => { self.advance(2); return Token::new(TokenKind::Gte, start_line, start_col); }
                "&&" => { self.advance(2); return Token::new(TokenKind::And, start_line, start_col); }
                "||" => { self.advance(2); return Token::new(TokenKind::Or, start_line, start_col); }
                "->" => { self.advance(2); return Token::new(TokenKind::Arrow, start_line, start_col); }
                "::" => { self.advance(2); return Token::new(TokenKind::DoubleColon, start_line, start_col); }
                _ => {}
            }
        }

        // Single-character tokens
        self.advance(1);
        match ch {
            '+' => Token::new(TokenKind::Plus, start_line, start_col),
            '-' => Token::new(TokenKind::Minus, start_line, start_col),
            '*' => Token::new(TokenKind::Star, start_line, start_col),
            '/' => Token::new(TokenKind::Slash, start_line, start_col),
            '%' => Token::new(TokenKind::Percent, start_line, start_col),
            '<' => Token::new(TokenKind::Lt, start_line, start_col),
            '>' => Token::new(TokenKind::Gt, start_line, start_col),
            '!' => Token::new(TokenKind::Not, start_line, start_col),
            '=' => Token::new(TokenKind::Assign, start_line, start_col),
            '(' => Token::new(TokenKind::LParen, start_line, start_col),
            ')' => Token::new(TokenKind::RParen, start_line, start_col),
            '{' => Token::new(TokenKind::LBrace, start_line, start_col),
            '}' => Token::new(TokenKind::RBrace, start_line, start_col),
            '[' => Token::new(TokenKind::LBracket, start_line, start_col),
            ']' => Token::new(TokenKind::RBracket, start_line, start_col),
            ',' => Token::new(TokenKind::Comma, start_line, start_col),
            ':' => Token::new(TokenKind::Colon, start_line, start_col),
            ';' => Token::new(TokenKind::Semicolon, start_line, start_col),
            '.' => Token::new(TokenKind::Dot, start_line, start_col),
            '?' => Token::new(TokenKind::Question, start_line, start_col),
            _ => panic!("Unexpected character '{}' at {}:{}", ch, self.filename, start_line),
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.source.len() {
            let ch = self.source[self.pos];
            
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
                self.pos += 1;
            } else if ch == '\r' {
                self.pos += 1;
                if self.pos < self.source.len() && self.source[self.pos] == '\n' {
                    self.pos += 1;
                }
                self.line += 1;
                self.column = 1;
            } else if ch == '\t' || ch == ' ' {
                self.column += 1;
                self.pos += 1;
            } else if ch == '/' && self.pos + 1 < self.source.len() && self.source[self.pos + 1] == '/' {
                // Line comment
                while self.pos < self.source.len() && self.source[self.pos] != '\n' {
                    self.pos += 1;
                }
            } else if ch == '/' && self.pos + 1 < self.source.len() && self.source[self.pos + 1] == '*' {
                // Block comment
                self.pos += 2;
                self.column += 2;
                while self.pos + 1 < self.source.len() {
                    if self.source[self.pos] == '*' && self.source[self.pos + 1] == '/' {
                        self.pos += 2;
                        self.column += 2;
                        break;
                    }
                    if self.source[self.pos] == '\n' {
                        self.line += 1;
                        self.column = 1;
                    } else {
                        self.column += 1;
                    }
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self, start_line: usize, start_col: usize) -> Token {
        self.advance(1); // Skip opening quote
        let mut value = String::new();
        
        while self.pos < self.source.len() && self.source[self.pos] != '"' {
            if self.source[self.pos] == '\\' {
                self.advance(1);
                if self.pos < self.source.len() {
                    match self.source[self.pos] {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        _ => value.push(self.source[self.pos]),
                    }
                    self.advance(1);
                }
            } else {
                value.push(self.source[self.pos]);
                self.advance(1);
            }
        }
        
        if self.pos >= self.source.len() {
            panic!("Unterminated string at {}:{}", self.filename, start_line);
        }
        
        self.advance(1); // Skip closing quote
        Token::new(TokenKind::String(value), start_line, start_col)
    }

    fn read_number(&mut self, start_line: usize, start_col: usize) -> Token {
        let mut value = String::new();
        let mut is_float = false;
        
        while self.pos < self.source.len() && (self.source[self.pos].is_ascii_digit() || self.source[self.pos] == '.') {
            if self.source[self.pos] == '.' {
                if is_float {
                    break;
                }
                is_float = true;
            }
            value.push(self.source[self.pos]);
            self.advance(1);
        }
        
        if is_float {
            Token::new(TokenKind::Float(value.parse().unwrap()), start_line, start_col)
        } else {
            Token::new(TokenKind::Integer(value.parse().unwrap()), start_line, start_col)
        }
    }

    fn read_identifier(&mut self, start_line: usize, start_col: usize) -> Token {
        let mut value = String::new();
        
        while self.pos < self.source.len() && (self.source[self.pos].is_ascii_alphanumeric() || self.source[self.pos] == '_') {
            value.push(self.source[self.pos]);
            self.advance(1);
        }
        
        let kind = match value.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "print" => TokenKind::Print,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "domain" => TokenKind::Domain,
            "spawn" => TokenKind::Spawn,
            "await" => TokenKind::Await,
            "chan" => TokenKind::Chan,
            "new" => TokenKind::New,
            "Some" => TokenKind::Some,
            "None" => TokenKind::None,
            "maybe" => TokenKind::Maybe,
            "type" => TokenKind::Type,
            "as" => TokenKind::As,
            "in" => TokenKind::In,
            "loop" => TokenKind::Loop,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            _ => TokenKind::Identifier(value),
        };
        
        Token::new(kind, start_line, start_col)
    }

    fn advance(&mut self, count: usize) {
        for _ in 0..count {
            if self.pos < self.source.len() {
                if self.source[self.pos] == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                self.pos += 1;
            }
        }
    }
}
