use std::collections::HashMap;
use std::fmt::Display;
use std::{cell::RefCell, rc::Rc};

use crate::token::TokenType;
use crate::{
    environment::Environment,
    interpreter::{Exit, Interpreter},
    stmt,
    token::{LiteralType, Token},
};

pub enum Callable {
    Function(LoxFunction),
    Class(LoxClass),
    Instance(Rc<RefCell<LoxInstance>>),
}

impl std::fmt::Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Callable")
    }
}

impl Clone for Callable {
    fn clone(&self) -> Self {
        match self {
            Callable::Function(lox_function) => Callable::Function(lox_function.clone()),
            Callable::Class(class) => Callable::Class(class.clone()),
            Callable::Instance(ins) => Callable::Instance(ins.clone()),
        }
    }
}

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
pub struct LoxFunction {
    declaration: Box<stmt::Function>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        declaration: stmt::Function,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            declaration: Box::new(declaration),
            closure,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: Rc<RefCell<LoxInstance>>) -> LoxFunction {
        let environment = Rc::new(RefCell::new(Environment::new_with_enclosing(Rc::clone(
            &self.closure,
        ))));
        environment.borrow_mut().define(
            "self".to_string(),
            LiteralType::Callable(Callable::Instance(instance)),
        );
        LoxFunction {
            declaration: self.declaration.clone(),
            closure: environment,
            is_initializer: self.is_initializer,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralType],
    ) -> Result<LiteralType, Exit> {
        let mut env = Environment::new_with_enclosing(Rc::clone(&self.closure));
        for (param, arg) in self.declaration.params.iter().zip(arguments) {
            env.define(param.lexeme.clone(), arg.clone());
        }

        let i = interpreter.execute_block(&self.declaration.body, env);

        match &i {
            Ok(_) => (),
            Err(e) => {
                if let Exit::Return(r) = e {
                    return Ok(r.clone());
                } else {
                    return Err(Exit::RuntimeError);
                }
            }
        }
        if self.is_initializer {
            return self.closure.borrow().get_at(
                0,
                &Token {
                    token_type: TokenType::SelfKW,
                    lexeme: String::from("self"),
                    literal: LiteralType::Nil,
                    line: self.declaration.name.line,
                },
            );
        }
        Ok(LiteralType::Nil)
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

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Box<LoxClass>>,
    pub methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<LoxClass>,
        methods: HashMap<String, LoxFunction>,
    ) -> Self {
        LoxClass {
            name,
            superclass: superclass.map(Box::new),
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        let f = self.methods.get(name);
        if f.is_some() {
            f
        } else {
            if let Some(sc) = &self.superclass {
                sc.find_method(name)
            } else {
                None
            }
        }
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("new") {
            initializer.arity()
        } else {
            0
        }
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[LiteralType],
    ) -> Result<LiteralType, Exit> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(Rc::new(self.clone()))));

        if let Some(initializer) = self.find_method("new") {
            initializer
                .bind(Rc::clone(&instance))
                .call(interpreter, args)?;
        }

        Ok(LiteralType::Callable(Callable::Instance(Rc::clone(
            &instance,
        ))))
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug)]
pub struct LoxInstance {
    pub class: Rc<LoxClass>,
    pub fields: HashMap<String, LiteralType>,
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        LoxInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<LiteralType, Exit> {
        if self.fields.contains_key(&name.lexeme) {
            Ok(self.fields.get(&name.lexeme).unwrap().clone())
        } else if let Some(method) = self.class.find_method(&name.lexeme) {
            Ok(LiteralType::Callable(Callable::Function(
                method.bind(Rc::new(RefCell::new(self.to_owned()))),
            )))
        } else {
            println!("Fields set: {:#?}", self.fields);
            crate::report(
                name.line,
                "",
                &format!("Undefined property {}.", name.lexeme),
            );
            Err(Exit::RuntimeError)
        }
    }

    pub fn get_or_nil(&self, name: &Token) -> Result<LiteralType, Exit> {
        if self.fields.contains_key(&name.lexeme) {
            Ok(self.fields.get(&name.lexeme).unwrap().clone())
        } else {
            Ok(LiteralType::Nil)
        }
    }

    pub fn set(&mut self, name: &Token, value: &LiteralType) {
        self.fields.insert(name.lexeme.clone(), value.clone());
    }
}
