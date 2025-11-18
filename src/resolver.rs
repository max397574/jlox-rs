use std::collections::HashMap;

use crate::{
    expr::{self, Expr},
    interpreter::Interpreter,
    parser::ParseError,
    stmt::{self, Stmt},
    token::Token,
};

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy, PartialEq)]
enum ClassType {
    None,
    Class,
}

impl Resolver<'_> {
    pub fn new(interpreter: &mut Interpreter) -> Resolver<'_> {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_statements(&mut self, statements: &Vec<Stmt>) -> Result<(), ParseError> {
        for stmt in statements {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), ParseError> {
        stmt.accept(self)?;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), ParseError> {
        expr.accept(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<(), ParseError> {
        if !self.scopes.is_empty() {
            if self.scopes.last().unwrap().contains_key(&name.lexeme) {
                crate::error(
                    name.line,
                    "Already a variable with this name in this scope.",
                );
                return Err(ParseError {});
            }
            self.scopes
                .last_mut()
                .unwrap()
                .insert(name.lexeme.clone(), false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if !self.scopes.is_empty() {
            self.scopes
                .last_mut()
                .unwrap()
                .insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: &stmt::Function,
        function_type: FunctionType,
    ) -> Result<(), ParseError> {
        let enclosing_fn = self.current_function;
        self.current_function = function_type;
        self.begin_scope();
        for param in function.params.iter() {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statements(&function.body)?;
        self.end_scope();
        self.current_function = enclosing_fn;
        Ok(())
    }
}

impl expr::Visitor<Result<(), ParseError>> for Resolver<'_> {
    fn visit_variable(&mut self, expr: &expr::Variable) -> Result<(), ParseError> {
        if !self.scopes.is_empty()
            && self.scopes.last().unwrap().get(&expr.name.lexeme) == Some(&false)
        {
            crate::error(
                expr.name.line,
                "Can't read local variable in its own initializer.",
            );
            return Err(ParseError {});
        }
        self.resolve_local(&Expr::Variable(expr.clone()), expr.name.clone());
        Ok(())
    }

    fn visit_assignment(&mut self, expr: &expr::Assignment) -> Result<(), ParseError> {
        self.resolve_expr(&expr.value)?;
        self.resolve_local(&Expr::Assignment(expr.clone()), expr.name.clone());
        Ok(())
    }

    fn visit_binary(&mut self, expr: &expr::Binary) -> Result<(), ParseError> {
        self.resolve_expr(&expr.left)?;
        self.resolve_expr(&expr.right)?;
        Ok(())
    }

    fn visit_call(&mut self, expr: &expr::Call) -> Result<(), ParseError> {
        self.resolve_expr(&expr.callee)?;
        for arg in &expr.arguments {
            self.resolve_expr(arg)?;
        }
        Ok(())
    }

    fn visit_grouping(&mut self, expr: &expr::Grouping) -> Result<(), ParseError> {
        self.resolve_expr(&expr.expr)?;
        Ok(())
    }

    fn visit_literal(&self, _: &expr::Literal) -> Result<(), ParseError> {
        Ok(())
    }

    fn visit_logical(&mut self, expr: &expr::Logical) -> Result<(), ParseError> {
        self.resolve_expr(&expr.left)?;
        self.resolve_expr(&expr.right)?;
        Ok(())
    }

    fn visit_unary(&mut self, expr: &expr::Unary) -> Result<(), ParseError> {
        self.resolve_expr(&expr.right)?;
        Ok(())
    }

    fn visit_get(&mut self, expr: &expr::Get) -> Result<(), ParseError> {
        self.resolve_expr(&expr.object)?;
        Ok(())
    }

    fn visit_set(&mut self, expr: &expr::Set) -> Result<(), ParseError> {
        self.resolve_expr(&expr.value)?;
        self.resolve_expr(&expr.object)?;
        Ok(())
    }

    fn visit_self_expr(&mut self, expr: &expr::SelfExpr) -> Result<(), ParseError> {
        if let ClassType::None = self.current_class {
            crate::error(expr.keyword.line, "Can't use 'this' ouside of a class.");
            return Err(ParseError {});
        }
        self.resolve_local(&Expr::SelfExpr(expr.clone()), expr.keyword.clone());
        Ok(())
    }
}

impl stmt::Visitor<Result<(), ParseError>> for Resolver<'_> {
    fn visit_block(&mut self, stmt: &stmt::Block) -> Result<(), ParseError> {
        self.begin_scope();
        self.resolve_statements(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_class(&mut self, stmt: &stmt::Class) -> Result<(), ParseError> {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;
        self.declare(&stmt.name)?;
        self.define(&stmt.name);

        self.begin_scope();
        self.scopes
            .last_mut()
            .unwrap()
            .insert(String::from("self"), true);

        for method in stmt.methods.iter() {
            if let Stmt::Function(method) = method {
                let declaration = if method.name.lexeme == "new" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                };

                self.resolve_function(method, declaration)?;
            }
        }

        self.end_scope();

        self.current_class = enclosing_class;

        Ok(())
    }

    fn visit_expression(&mut self, stmt: &stmt::Expression) -> Result<(), ParseError> {
        self.resolve_expr(&stmt.expr)?;
        Ok(())
    }

    fn visit_if(&mut self, stmt: &stmt::If) -> Result<(), ParseError> {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(&stmt.then_branch)?;
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_var(&mut self, stmt: &stmt::Var) -> Result<(), ParseError> {
        self.declare(&stmt.name)?;
        self.resolve_expr(&stmt.initializer)?;
        self.define(&stmt.name);

        Ok(())
    }

    fn visit_while(&mut self, stmt: &stmt::While) -> Result<(), ParseError> {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(&stmt.body)?;
        Ok(())
    }

    fn visit_function(&mut self, stmt: &stmt::Function) -> Result<(), ParseError> {
        self.declare(&stmt.name)?;
        self.define(&stmt.name);

        self.resolve_function(stmt, FunctionType::Function)?;
        Ok(())
    }

    fn visit_return(&mut self, stmt: &stmt::Return) -> Result<(), ParseError> {
        if let FunctionType::None = self.current_function {
            crate::error(
                stmt.keyword.line,
                "Can't return without enclosing function!",
            );
            return Err(ParseError {});
        } else if let FunctionType::Initializer = self.current_function {
            crate::error(stmt.keyword.line, "Can't return from an initializer!");
            return Err(ParseError {});
        }
        self.resolve_expr(&stmt.value)?;
        Ok(())
    }
}
