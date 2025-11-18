use crate::{
    environment::Environment,
    expr::{self, Binary, Expr, Grouping, Literal, Logical, Unary},
    lox_callable::{Callable, LoxCallable, LoxClass, LoxFunction, NativeFunction},
    report,
    stmt::{self, Expression, Stmt},
    token::{LiteralType, Token, TokenType},
};

use std::{cell::RefCell, collections::HashMap, time::SystemTime};
use std::{rc::Rc, time::UNIX_EPOCH};

#[derive(Debug)]
pub enum Exit {
    RuntimeError,
    Return(LiteralType),
}

pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
    pub globals: Rc<RefCell<Environment>>,
    locals: HashMap<Expr, usize>,
}

impl expr::Visitor<Result<LiteralType, Exit>> for Interpreter {
    fn visit_binary(&mut self, expr: &Binary) -> Result<LiteralType, Exit> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;
        match expr.operator.token_type {
            TokenType::Minus => {
                if let LiteralType::Number(l_val) = left
                    && let LiteralType::Number(r_val) = right
                {
                    Ok(LiteralType::Number(l_val - r_val))
                } else {
                    Err(Exit::RuntimeError)
                }
            }
            TokenType::Slash => {
                if let LiteralType::Number(l_val) = left
                    && let LiteralType::Number(r_val) = right
                {
                    Ok(LiteralType::Number(l_val / r_val))
                } else {
                    report(
                        expr.operator.line,
                        "",
                        &format!("Both operands of '/' must be a number, got {left}, {right}"),
                    );
                    Err(Exit::RuntimeError)
                }
            }
            TokenType::Percentage => {
                if let LiteralType::Number(l_val) = left
                    && let LiteralType::Number(r_val) = right
                {
                    Ok(LiteralType::Number(l_val % r_val))
                } else {
                    report(
                        expr.operator.line,
                        "",
                        &format!("Both operands of '%' must be a number, got {left}, {right}"),
                    );
                    Err(Exit::RuntimeError)
                }
            }
            TokenType::Star => {
                if let LiteralType::Number(l_val) = left
                    && let LiteralType::Number(r_val) = right
                {
                    Ok(LiteralType::Number(l_val * r_val))
                } else {
                    report(
                        expr.operator.line,
                        "",
                        &format!("Both operands of '*' must be a number, got {left}, {right}"),
                    );
                    Err(Exit::RuntimeError)
                }
            }
            TokenType::Plus => {
                if let LiteralType::Number(l_val) = left
                    && let LiteralType::Number(r_val) = right
                {
                    Ok(LiteralType::Number(l_val + r_val))
                } else if let LiteralType::String(l_val) = &left
                    && let LiteralType::String(r_val) = right
                {
                    Ok(LiteralType::String(format!("{l_val}{r_val}")))
                } else {
                    report(
                        expr.operator.line,
                        "",
                        &format!(
                            "Both operands of '*' must be a number or string, got {left}, {right}"
                        ),
                    );
                    Err(Exit::RuntimeError)
                }
            }
            TokenType::Greater => Ok(LiteralType::Boolean(match (&left, &right) {
                (LiteralType::Number(l_val), LiteralType::Number(r_val)) => l_val > r_val,
                _ => {
                    report(
                        expr.operator.line,
                        "",
                        &format!("Can't compare {left}, {right}"),
                    );

                    return Err(Exit::RuntimeError);
                }
            })),
            TokenType::Less => Ok(LiteralType::Boolean(match (&left, &right) {
                (LiteralType::Number(l_val), LiteralType::Number(r_val)) => l_val < r_val,
                _ => {
                    report(
                        expr.operator.line,
                        "",
                        &format!("Can't compare {left}, {right}"),
                    );

                    return Err(Exit::RuntimeError);
                }
            })),
            TokenType::GreaterEqual => Ok(LiteralType::Boolean(match (&left, &right) {
                (LiteralType::Number(l_val), LiteralType::Number(r_val)) => l_val >= r_val,
                _ => {
                    report(
                        expr.operator.line,
                        "",
                        &format!("Can't compare {left}, {right}"),
                    );

                    return Err(Exit::RuntimeError);
                }
            })),
            TokenType::LessEqual => Ok(LiteralType::Boolean(match (&left, &right) {
                (LiteralType::Number(l_val), LiteralType::Number(r_val)) => l_val <= r_val,
                _ => {
                    report(
                        expr.operator.line,
                        "",
                        &format!("Can't compare {left}, {right}"),
                    );

                    return Err(Exit::RuntimeError);
                }
            })),
            TokenType::EqualEqual => Ok(LiteralType::Boolean(self.is_equal(&left, &right))),
            TokenType::BangEqual => Ok(LiteralType::Boolean(!self.is_equal(&left, &right))),

            _ => unreachable!(),
        }
    }

    fn visit_call(&mut self, expr: &expr::Call) -> Result<LiteralType, Exit> {
        let callee = self.evaluate(&expr.callee)?;
        let mut arguments = Vec::new();
        for argument in expr.arguments.iter() {
            arguments.push(self.evaluate(argument)?);
        }

        match callee {
            LiteralType::NativeFunction(func) => {
                func.check_arity(arguments.len(), &expr.paren)?;
                func.call(self, &arguments)
            }
            LiteralType::Callable(Callable::Function(func)) => {
                func.check_arity(arguments.len(), &expr.paren)?;
                func.call(self, &arguments)
            }
            LiteralType::Callable(Callable::Class(class)) => {
                class.check_arity(arguments.len(), &expr.paren)?;
                class.call(self, &arguments)
            }
            _ => {
                report(expr.paren.line, "", "Can only call functions/methods");
                Err(Exit::RuntimeError)
            }
        }
    }

    fn visit_grouping(&mut self, expr: &Grouping) -> Result<LiteralType, Exit> {
        self.evaluate(&expr.expr)
    }

    fn visit_literal(&self, expr: &Literal) -> Result<LiteralType, Exit> {
        Ok(expr.value.clone())
    }

    fn visit_logical(&mut self, expr: &Logical) -> Result<LiteralType, Exit> {
        let left = self.evaluate(&expr.left)?;

        if matches!(expr.operator.token_type, TokenType::Or | TokenType::BarBar) {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else if !self.is_truthy(&left) {
            return Ok(left);
        }

        self.evaluate(&expr.right)
    }

    fn visit_unary(&mut self, expr: &Unary) -> Result<LiteralType, Exit> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.token_type {
            TokenType::Minus => match right {
                LiteralType::Number(val) => Ok(LiteralType::Number(-val)),
                val => {
                    report(
                        expr.operator.line,
                        "",
                        &format!("Operand of '-' must be a number, got {val}"),
                    );
                    Err(Exit::RuntimeError)
                }
            },
            TokenType::Bang => Ok(LiteralType::Boolean(!self.is_truthy(&right))),
            _ => unreachable!(),
        }
    }

    fn visit_variable(&mut self, expr: &expr::Variable) -> Result<LiteralType, Exit> {
        self.look_up_variable(&expr.name, &Expr::Variable(expr.clone()))
    }

    fn visit_assignment(&mut self, expr: &expr::Assignment) -> Result<LiteralType, Exit> {
        let value = self.evaluate(&expr.value)?;
        let distance = self.locals.get(&Expr::Assignment(expr.clone()));
        if let Some(d) = distance {
            self.environment
                .borrow_mut()
                .assign_at(*d, expr.name.clone(), value.clone());
        } else {
            self.globals
                .borrow_mut()
                .assign(&expr.name, value.clone())?;
        }
        Ok(value)
    }

    fn visit_get(&mut self, expr: &expr::Get) -> Result<LiteralType, Exit> {
        let object = self.evaluate(&expr.object)?;
        if let LiteralType::Callable(Callable::Instance(instance)) = object {
            instance.borrow().get(&expr.name)
        } else {
            report(expr.name.line, "", "Only instances have properties.");
            Err(Exit::RuntimeError)
        }
    }

    fn visit_set(&mut self, expr: &expr::Set) -> Result<LiteralType, Exit> {
        let object = self.evaluate(&expr.object)?;
        if let LiteralType::Callable(Callable::Instance(instance)) = object {
            let value = self.evaluate(&expr.value)?;
            instance.borrow_mut().set(&expr.name, &value);
            Ok(value)
        } else {
            report(expr.name.line, "", "Only instances have fields.");
            Err(Exit::RuntimeError)
        }
    }

    fn visit_self_expr(&mut self, expr: &expr::SelfExpr) -> Result<LiteralType, Exit> {
        self.look_up_variable(&expr.keyword, &Expr::SelfExpr(expr.clone()))
    }
}

impl stmt::Visitor<Result<(), Exit>> for Interpreter {
    fn visit_expression(&mut self, stmt: &Expression) -> Result<(), Exit> {
        self.evaluate(&stmt.expr)?;
        Ok(())
    }

    fn visit_if(&mut self, stmt: &stmt::If) -> Result<(), Exit> {
        let cond = &self.evaluate(&stmt.condition)?;
        if self.is_truthy(cond) {
            self.execute(&stmt.then_branch)?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)?;
        }
        Ok(())
    }

    fn visit_while(&mut self, stmt: &stmt::While) -> Result<(), Exit> {
        loop {
            let eval = self.evaluate(&stmt.condition)?;
            if !self.is_truthy(&eval) {
                break;
            }
            self.execute(&stmt.body)?;
        }
        Ok(())
    }

    fn visit_var(&mut self, stmt: &stmt::Var) -> Result<(), Exit> {
        let val = self.evaluate(&stmt.initializer)?;
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), val);
        Ok(())
    }

    fn visit_block(&mut self, block: &stmt::Block) -> Result<(), Exit> {
        self.execute_block(
            &block.statements,
            Environment::new_with_enclosing(self.environment.clone()),
        )
    }

    fn visit_function(&mut self, stmt: &stmt::Function) -> Result<(), Exit> {
        let function = LoxFunction::new(stmt.clone(), self.environment.clone(), false);
        self.environment.borrow_mut().define(
            stmt.name.lexeme.clone(),
            LiteralType::Callable(Callable::Function(function)),
        );
        Ok(())
    }

    fn visit_class(&mut self, stmt: &stmt::Class) -> Result<(), Exit> {
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), LiteralType::Nil);

        let mut methods = HashMap::new();
        for method in stmt.methods.iter() {
            if let Stmt::Function(method) = method {
                let function = LoxFunction::new(
                    method.clone(),
                    Rc::clone(&self.environment),
                    method.name.lexeme == "new",
                );
                methods.insert(method.name.lexeme.clone(), function);
            }
        }

        let class = LoxClass::new(stmt.name.lexeme.clone(), methods);
        self.environment
            .borrow_mut()
            .assign(&stmt.name, LiteralType::Callable(Callable::Class(class)))?;

        Ok(())
    }

    fn visit_return(&mut self, expr: &stmt::Return) -> Result<(), Exit> {
        Err(Exit::Return(self.evaluate(&expr.value)?))
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        // Std functions
        globals.borrow_mut().define(
            String::from("clock"),
            LiteralType::NativeFunction(NativeFunction {
                arity: 0,
                callable: |_, _| {
                    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    LiteralType::Number(timestamp.as_millis() as f64)
                },
            }),
        );
        globals.borrow_mut().define(
            String::from("print"),
            LiteralType::NativeFunction(NativeFunction {
                arity: 1,
                callable: |_, args| {
                    println!("{}", args[0]);
                    LiteralType::Nil
                },
            }),
        );
        Self {
            environment: globals.clone(),
            globals,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), Exit> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), Exit> {
        stmt.accept(self)?;
        Ok(())
    }

    pub fn execute_block(&mut self, statements: &[Stmt], env: Environment) -> Result<(), Exit> {
        let previous = Rc::clone(&self.environment);
        self.environment = Rc::new(RefCell::new(env));

        let result = statements.iter().try_for_each(|stat| self.execute(stat));

        self.environment = previous;
        result
    }

    fn is_equal(&mut self, left: &LiteralType, right: &LiteralType) -> bool {
        match (&left, &right) {
            (LiteralType::Number(l_val), LiteralType::Number(r_val)) => l_val == r_val,
            (LiteralType::String(l_val), LiteralType::String(r_val)) => l_val == r_val,
            (LiteralType::Boolean(l_val), LiteralType::Boolean(r_val)) => l_val == r_val,
            (LiteralType::Nil, LiteralType::Nil) => true,
            _ => false,
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<LiteralType, Exit> {
        expr.accept(self)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.clone(), depth);
    }

    pub fn look_up_variable(&mut self, name: &Token, expr: &Expr) -> Result<LiteralType, Exit> {
        let distance = self.locals.get(expr);
        if let Some(d) = distance {
            self.environment.borrow_mut().get_at(*d, name)
        } else {
            self.globals.borrow().get(name)
        }
    }

    /// Everything but false and nil is truthy
    fn is_truthy(&mut self, expr: &LiteralType) -> bool {
        match expr {
            LiteralType::String(_) => true,
            LiteralType::Number(_) => true,
            LiteralType::Nil => false,
            LiteralType::Boolean(val) => *val,
            LiteralType::Callable(_) => true,
            LiteralType::NativeFunction(_) => true,
        }
    }
}
