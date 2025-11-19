use crate::{
    interpreter::Exit,
    report,
    token::{LiteralType, Token},
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, LiteralType>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: LiteralType) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: LiteralType) -> Result<(), Exit> {
        #[allow(clippy::map_entry)]
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else {
            if let Some(enclosing) = &self.enclosing {
                return enclosing.borrow_mut().assign(name, value);
            }
            report(
                name.line,
                "",
                &format!("Assigning to undefinied variable {}.", name.lexeme),
            );
            Err(Exit::RuntimeError)
        }
    }

    pub fn get(&self, name: &Token) -> Result<LiteralType, Exit> {
        if self.values.contains_key(&name.lexeme) {
            Ok(self.values.get(&name.lexeme).unwrap().clone())
        } else {
            if let Some(enclosing) = &self.enclosing {
                return enclosing.borrow_mut().get(name);
            }
            report(
                name.line,
                "",
                &format!("Undefinied variable {}.", name.lexeme),
            );
            Err(Exit::RuntimeError)
        }
    }

    pub fn get_at(&self, distance: usize, name: &Token) -> Result<LiteralType, Exit> {
        if distance == 0 {
            self.get(name)
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow()
                .get_at(distance - 1, name)
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: Token, value: LiteralType) {
        if distance == 0 {
            self.define(name.lexeme, value);
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow_mut()
                .assign_at(distance - 1, name, value);
        }
    }
}
