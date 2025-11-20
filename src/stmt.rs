use crate::{expr::Expr, token::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Block),
    Class(Class),
    Expression(Expression),
    If(If),
    Var(Var),
    While(While),
    Function(Function),
    Return(Return),
}

#[derive(Debug, Clone)]
pub struct Return {
    pub keyword: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Token,
    pub superclass: Option<Expr>,
    pub methods: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<Expr>,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: Token,
    pub initializer: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct While {
    pub condition: Box<Expr>,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub expr: Box<Expr>,
}

pub trait Visitor<T> {
    fn visit_block(&mut self, stmt: &Block) -> T;
    fn visit_class(&mut self, stmt: &Class) -> T;
    fn visit_expression(&mut self, stmt: &Expression) -> T;
    fn visit_if(&mut self, stmt: &If) -> T;
    fn visit_var(&mut self, stmt: &Var) -> T;
    fn visit_while(&mut self, stmt: &While) -> T;
    fn visit_function(&mut self, stmt: &Function) -> T;
    fn visit_return(&mut self, expr: &Return) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Stmt::Block(block) => visitor.visit_block(block),
            Stmt::Class(class) => visitor.visit_class(class),
            Stmt::Expression(expression) => visitor.visit_expression(expression),
            Stmt::If(if_stmt) => visitor.visit_if(if_stmt),
            Stmt::Var(print) => visitor.visit_var(print),
            Stmt::While(while_stmt) => visitor.visit_while(while_stmt),
            Stmt::Function(func) => visitor.visit_function(func),
            Stmt::Return(ret) => visitor.visit_return(ret),
        }
    }
}
