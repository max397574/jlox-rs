use crate::lox_callable::{Function, NativeFunction};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: LiteralType,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: LiteralType, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.token_type, self.lexeme, self.literal)
    }
}

#[derive(Debug, Clone)]
pub enum LiteralType {
    String(String),
    Number(f64),
    Nil,
    Boolean(bool),
    Function(Function),
    NativeFunction(NativeFunction),
}

impl Display for LiteralType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralType::String(val) => write!(f, "{val}"),
            LiteralType::Number(val) => write!(f, "{val}"),
            LiteralType::Nil => write!(f, "nil"),
            LiteralType::Boolean(val) => write!(f, "\"{val}\""),
            LiteralType::Function(_) => write!(f, "<fn>"),
            LiteralType::NativeFunction(_) => write!(f, "<native fn>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Percentage,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    AmperAmper,
    BarBar,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::BarBar => write!(f, "||"),
            TokenType::AmperAmper => write!(f, "&&"),
            TokenType::LeftParen => write!(f, "LeftParen"),
            TokenType::RightParen => write!(f, "RightParen"),
            TokenType::LeftBrace => write!(f, "LeftBrace"),
            TokenType::RightBrace => write!(f, "RightBrace"),
            TokenType::Comma => write!(f, "Comma"),
            TokenType::Dot => write!(f, "Dot"),
            TokenType::Minus => write!(f, "Minus"),
            TokenType::Plus => write!(f, "Plus"),
            TokenType::Semicolon => write!(f, "Semicolon"),
            TokenType::Slash => write!(f, "Slash"),
            TokenType::Star => write!(f, "Star"),
            TokenType::Percentage => write!(f, "Percentage"),
            TokenType::Bang => write!(f, "Bang"),
            TokenType::BangEqual => write!(f, "BangEqual"),
            TokenType::Equal => write!(f, "Equal"),
            TokenType::EqualEqual => write!(f, "EqualEqual"),
            TokenType::Greater => write!(f, "Greater"),
            TokenType::GreaterEqual => write!(f, "GreaterEqual"),
            TokenType::Less => write!(f, "Less"),
            TokenType::LessEqual => write!(f, "LessEqual"),
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::String => write!(f, "String"),
            TokenType::Number => write!(f, "Number"),
            TokenType::And => write!(f, "And"),
            TokenType::Class => write!(f, "Class"),
            TokenType::Else => write!(f, "Else"),
            TokenType::False => write!(f, "False"),
            TokenType::Fun => write!(f, "Fun"),
            TokenType::For => write!(f, "For"),
            TokenType::If => write!(f, "If"),
            TokenType::Nil => write!(f, "Nil"),
            TokenType::Or => write!(f, "Or"),
            TokenType::Return => write!(f, "Return"),
            TokenType::Super => write!(f, "Super"),
            TokenType::This => write!(f, "This"),
            TokenType::True => write!(f, "True"),
            TokenType::Var => write!(f, "Var"),
            TokenType::While => write!(f, "While"),
            TokenType::Eof => write!(f, "Eof"),
        }
    }
}
