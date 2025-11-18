use crate::{
    error,
    token::{LiteralType, Token, TokenType},
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> u8 {
        let c = self.source.as_bytes()[self.current];
        self.current += 1;
        c
    }

    fn peek(&self) -> u8 {
        self.source.as_bytes()[self.current]
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() {
            0
        } else {
            self.source.as_bytes()[self.current + 1]
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            b'(' => self.add_token(TokenType::LeftParen, LiteralType::Nil),
            b')' => self.add_token(TokenType::RightParen, LiteralType::Nil),
            b'{' => self.add_token(TokenType::LeftBrace, LiteralType::Nil),
            b'}' => self.add_token(TokenType::RightBrace, LiteralType::Nil),
            b',' => self.add_token(TokenType::Comma, LiteralType::Nil),
            b'.' => self.add_token(TokenType::Dot, LiteralType::Nil),
            b'-' => self.add_token(TokenType::Minus, LiteralType::Nil),
            b'+' => self.add_token(TokenType::Plus, LiteralType::Nil),
            b';' => self.add_token(TokenType::Semicolon, LiteralType::Nil),
            b'*' => self.add_token(TokenType::Star, LiteralType::Nil),
            b'%' => self.add_token(TokenType::Percentage, LiteralType::Nil),
            b'&' => {
                if self.expect_next(b'&') {
                    self.add_token(TokenType::AmperAmper, LiteralType::Nil);
                } else {
                    error(self.line, &format!("Unexpected Character {c}"));
                }
            }
            b'|' => {
                if self.expect_next(b'|') {
                    self.add_token(TokenType::BarBar, LiteralType::Nil);
                } else {
                    error(self.line, &format!("Unexpected Character {c}"));
                }
            }
            b'=' => {
                if self.expect_next(b'=') {
                    self.add_token(TokenType::EqualEqual, LiteralType::Nil);
                } else {
                    self.add_token(TokenType::Equal, LiteralType::Nil);
                }
            }
            b'!' => {
                if self.expect_next(b'=') {
                    self.add_token(TokenType::BangEqual, LiteralType::Nil);
                } else {
                    self.add_token(TokenType::Bang, LiteralType::Nil);
                }
            }
            b'<' => {
                if self.expect_next(b'=') {
                    self.add_token(TokenType::LessEqual, LiteralType::Nil);
                } else {
                    self.add_token(TokenType::Less, LiteralType::Nil);
                }
            }
            b'>' => {
                if self.expect_next(b'=') {
                    self.add_token(TokenType::GreaterEqual, LiteralType::Nil);
                } else {
                    self.add_token(TokenType::Greater, LiteralType::Nil);
                }
            }
            b'/' => {
                if self.expect_next(b'/') {
                    while !self.is_at_end() && (self.peek() != b'\n') {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, LiteralType::Nil);
                }
            }

            b'"' => self.string(b'"'),
            b'\'' => self.string(b'\''),

            b' ' | b'\t' | b'\r' => {}
            b'\n' => self.line += 1,
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_ascii_alphabetic() {
                    self.identifier();
                } else {
                    error(self.line, &format!("Unexpected Character {}", c as char));
                }
            }
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }

        let text = String::from(&self.source[self.start..self.current]);

        if let Some(token_type) = get_keyword(&text) {
            self.add_token(token_type, LiteralType::Nil);
        } else {
            self.add_token(TokenType::Identifier, LiteralType::String(text));
        }
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() && !self.is_at_end() {
            self.advance();
        }

        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() && !self.is_at_end() {
                self.advance();
            }
        }

        let text = String::from(&self.source[self.start..self.current]);
        self.add_token(
            TokenType::Number,
            LiteralType::Number(text.parse::<f64>().unwrap()),
        );
    }

    fn string(&mut self, delimiter: u8) {
        while self.peek() != delimiter && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(
                self.line,
                &format!("Unterminated string (missing {}", delimiter as char),
            );
        }

        // consume closing delimiter
        self.advance();

        let text = String::from(&self.source[self.start + 1..self.current - 1]);
        self.add_token(TokenType::String, LiteralType::String(text));
    }

    fn expect_next(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] == expected {
            self.current += 1;
            true
        } else {
            false
        }
    }

    fn add_token(&mut self, token_type: TokenType, literal: LiteralType) {
        let text = String::from(&self.source[self.start..self.current]);
        self.tokens
            .push(Token::new(token_type, text, literal, self.line));
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            LiteralType::Nil,
            self.line,
        ));
        self.tokens.clone()
    }
}

fn get_keyword(word: &str) -> Option<TokenType> {
    match word {
        "and" => Some(TokenType::And),
        "class" => Some(TokenType::Class),
        "else" => Some(TokenType::Else),
        "false" => Some(TokenType::False),
        "fun" => Some(TokenType::Fun),
        "for" => Some(TokenType::For),
        "if" => Some(TokenType::If),
        "nil" => Some(TokenType::Nil),
        "or" => Some(TokenType::Or),
        "return" => Some(TokenType::Return),
        "super" => Some(TokenType::Super),
        "self" => Some(TokenType::SelfKW),
        "true" => Some(TokenType::True),
        "var" => Some(TokenType::Var),
        "while" => Some(TokenType::While),
        _ => None,
    }
}
