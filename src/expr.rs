use crate::token::{LiteralType, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    Assignment(Assignment),
    Binary(Binary),
    Call(Call),
    Grouping(Grouping),
    Literal(Literal),
    Logical(Logical),
    Unary(Unary),
    Variable(Variable),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: LiteralType,
}

#[derive(Debug, Clone)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Literal {
    pub fn new(value: LiteralType) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

pub trait Visitor<T> {
    fn visit_assignment(&mut self, expr: &Assignment) -> T;
    fn visit_binary(&mut self, expr: &Binary) -> T;
    fn visit_call(&mut self, expr: &Call) -> T;
    fn visit_grouping(&mut self, expr: &Grouping) -> T;
    fn visit_literal(&self, expr: &Literal) -> T;
    fn visit_logical(&mut self, expr: &Logical) -> T;
    fn visit_unary(&mut self, expr: &Unary) -> T;
    fn visit_variable(&mut self, expr: &Variable) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Assignment(assignment) => visitor.visit_assignment(assignment),
            Expr::Binary(binary) => visitor.visit_binary(binary),
            Expr::Call(call) => visitor.visit_call(call),
            Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Logical(logical) => visitor.visit_logical(logical),
            Expr::Unary(unary) => visitor.visit_unary(unary),
            Expr::Variable(variable) => visitor.visit_variable(variable),
        }
    }
}
