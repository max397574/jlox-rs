mod environment;
mod expr;
mod interpreter;
mod lox_callable;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;

use std::io::Write;
use std::{fs, io};

use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;

pub fn run_file(arg: &str) {
    let content = fs::read_to_string(arg);
    run(&content.unwrap());
}

pub fn run_prompt() {
    loop {
        print!(">> ");
        let mut line = String::new();
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut line).unwrap();
        run(&line);
    }
}

fn run(content: &str) {
    let mut scanner = Scanner::new(content.to_owned());
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();

    if let Ok(stmts) = stmts {
        let mut interpreter = Interpreter::new();
        let mut resolver = Resolver::new(&mut interpreter);
        if resolver.resolve_statements(&stmts).is_err() {
            eprintln!("Parsing error while resolving");
            return;
        }
        if interpreter.interpret(&stmts).is_err() {
            eprintln!("Runtime Error");
        }
    } else {
        eprintln!("Parsing error");
    }
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

pub fn report(line: usize, location: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, location, message);
}
