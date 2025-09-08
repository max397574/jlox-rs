use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::{Exit, Interpreter},
    stmt::{self, Stmt},
    token::{LiteralType, Token},
};

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralType],
    ) -> Result<LiteralType, Exit>;
    fn arity(&self) -> usize;
    fn check_arity(&self, args_len: usize, current_token: &Token) -> Result<(), Exit> {
        if args_len != self.arity() {
            crate::report(
                current_token.line,
                "",
                &format!("Expected {} arguments but got {}.", self.arity(), args_len),
            );
            return Err(Exit::RuntimeError);
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    declaration: Box<stmt::Function>,
    closure: Rc<RefCell<Environment>>,
}

impl Function {
    pub fn new(declaration: stmt::Function, closure: Rc<RefCell<Environment>>) -> Self {
        Self {
            declaration: Box::new(declaration),
            closure,
        }
    }
}

impl LoxCallable for Function {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralType],
    ) -> Result<LiteralType, Exit> {
        let mut env = Environment::new_with_enclosing(self.closure.clone());
        for (param, arg) in self.declaration.params.iter().zip(arguments) {
            env.define(param.lexeme.clone(), arg.clone());
        }

        if let Err(exit) = interpreter.execute_block(&self.declaration.body, env) {
            if let Exit::Return(ret_val) = exit {
                Ok(ret_val)
            } else {
                Err(exit)
            }
        } else {
            Ok(LiteralType::Nil)
        }
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
}

#[derive(Clone, Debug)]
pub struct NativeFunction {
    pub arity: usize,
    pub callable: fn(&mut Interpreter, &[LiteralType]) -> LiteralType,
}

impl LoxCallable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[LiteralType],
    ) -> Result<LiteralType, Exit> {
        Ok((self.callable)(interpreter, args))
    }
}
