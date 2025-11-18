use crate::token::{LiteralType, Token};
use std::hash::Hash;

#[derive(Debug, Clone)]
pub enum Expr {
    Assignment(Assignment),
    Binary(Binary),
    Call(Call),
    Get(Get),
    Grouping(Grouping),
    Literal(Literal),
    Logical(Logical),
    Set(Set),
    Unary(Unary),
    SelfExpr(SelfExpr),
    Variable(Variable),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: Token,
    pub value: Box<Expr>,
    pub uuid: usize,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Token,
    pub uuid: usize,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
    pub uuid: usize,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
    pub uuid: usize,
}

#[derive(Debug, Clone)]
pub struct Get {
    pub object: Box<Expr>,
    pub name: Token,
    pub uuid: usize,
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expr: Box<Expr>,
    pub uuid: usize,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: LiteralType,
    pub uuid: usize,
}

#[derive(Debug, Clone)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
    pub uuid: usize,
}
#[derive(Debug, Clone)]
pub struct Set {
    pub object: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>,
    pub uuid: usize,
}

impl Literal {
    pub fn new(value: LiteralType, uuid: usize) -> Self {
        Self { value, uuid }
    }
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
    pub uuid: usize,
}

#[derive(Debug, Clone)]
pub struct SelfExpr {
    pub keyword: Token,
    pub uuid: usize,
}

pub trait Visitor<T> {
    fn visit_assignment(&mut self, expr: &Assignment) -> T;
    fn visit_binary(&mut self, expr: &Binary) -> T;
    fn visit_call(&mut self, expr: &Call) -> T;
    fn visit_get(&mut self, expr: &Get) -> T;
    fn visit_grouping(&mut self, expr: &Grouping) -> T;
    fn visit_literal(&self, expr: &Literal) -> T;
    fn visit_logical(&mut self, expr: &Logical) -> T;
    fn visit_unary(&mut self, expr: &Unary) -> T;
    fn visit_set(&mut self, expr: &Set) -> T;
    fn visit_self_expr(&mut self, expr: &SelfExpr) -> T;
    fn visit_variable(&mut self, expr: &Variable) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Assignment(assignment) => visitor.visit_assignment(assignment),
            Expr::Binary(binary) => visitor.visit_binary(binary),
            Expr::Call(call) => visitor.visit_call(call),
            Expr::Get(get) => visitor.visit_get(get),
            Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Logical(logical) => visitor.visit_logical(logical),
            Expr::Unary(unary) => visitor.visit_unary(unary),
            Expr::Set(set) => visitor.visit_set(set),
            Expr::SelfExpr(self_expr) => visitor.visit_self_expr(self_expr),
            Expr::Variable(variable) => visitor.visit_variable(variable),
        }
    }

    fn get_uid(&self) -> usize {
        match self {
            Expr::Assignment(e) => e.uuid,
            Expr::Binary(e) => e.uuid,
            Expr::Call(e) => e.uuid,
            Expr::Get(e) => e.uuid,
            Expr::Grouping(e) => e.uuid,
            Expr::Literal(e) => e.uuid,
            Expr::Logical(e) => e.uuid,
            Expr::Unary(e) => e.uuid,
            Expr::Set(e) => e.uuid,
            Expr::SelfExpr(e) => e.uuid,
            Expr::Variable(e) => e.uuid,
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.get_uid() == other.get_uid()
    }
}

impl Eq for Expr {}

impl Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // core::mem::discriminant(self).hash(state);
        self.get_uid().hash(state);
    }
}
