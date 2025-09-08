use crate::{
    expr::{self, Assignment, Binary, Expr, Grouping, Literal, Logical, Unary, Variable},
    stmt::{self, Block, Expression, Stmt, Var},
    token::{
        LiteralType, Token,
        TokenType::{self, *},
    },
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

pub struct ParseError {}

static mut UUID: usize = 0;

pub fn uuid_next() -> usize {
    unsafe {
        UUID += 1;
        UUID
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        let mut had_error = false;
        while !self.is_at_end() {
            if let Ok(s) = self.declaration() {
                statements.push(s);
            } else {
                had_error = true;
            }
        }
        if had_error {
            Err(ParseError {})
        } else {
            Ok(statements)
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == Semicolon {
                return;
            }

            match self.peek().token_type {
                Class | Fun | Var | For | If | While | Return => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        let res = if self.matches(&[Var]) {
            self.var_declaration()
        } else if self.matches(&[Fun]) {
            self.fun_declaration("function")
        } else {
            self.statement()
        };

        if res.is_ok() {
            res
        } else {
            self.synchronize();
            Err(ParseError {})
        }
    }

    fn fun_declaration(&mut self, kind: &str) -> Result<Stmt, ParseError> {
        let name = self.consume(&Identifier, &format!("Expected {kind} name"))?;
        self.consume(&LeftParen, &format!("Expected '(' after {kind} name"))?;
        let mut params = Vec::new();
        if !self.check(&RightParen) {
            loop {
                if params.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 parameters");
                }
                params.push(self.consume(&Identifier, "Expect parameter name.")?);
                if !self.matches(&[Comma]) {
                    break;
                }
            }
        }
        self.consume(&RightParen, "Expected ')' after parameters")?;
        self.consume(&LeftBrace, &format!("Expected '{{' before {kind} body"))?;
        let body = self.block()?;
        Ok(Stmt::Function(stmt::Function { name, params, body }))
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(&Identifier, "Expect variable name.")?;
        let mut initializer = Expr::Literal(Literal {
            value: LiteralType::Nil,
            uuid: uuid_next(),
        });

        if self.matches(&[Equal]) {
            initializer = self.expression()?;
        }

        self.consume(&Semicolon, "Expect semicolon.")?;

        Ok(Stmt::Var(Var {
            name,
            initializer: Box::new(initializer),
        }))
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.is_at_end() {
            return self.expression_statement();
        }
        self.advance();
        match self.previous().token_type {
            For => self.for_statement(),
            If => self.if_statement(),
            While => self.while_statement(),
            LeftBrace => Ok(Stmt::Block(Block {
                statements: self.block()?,
            })),
            Return => self.return_statement(),
            _ => {
                self.current -= 1;
                self.expression_statement()
            }
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous();
        let value = if !self.check(&Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(expr::Literal {
                value: LiteralType::Nil,
                uuid: uuid_next(),
            })
        };
        self.consume(&Semicolon, "Expect ';' after return.")?;
        Ok(Stmt::Return(stmt::Return {
            keyword,
            value: Box::new(value),
        }))
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(&LeftParen, "Expect '(' after 'for'.")?;
        let initializer = if self.matches(&[Semicolon]) {
            None
        } else if self.matches(&[Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition: Expr = if !self.check(&Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Literal {
                value: LiteralType::Boolean(true),
                uuid: uuid_next(),
            })
        };
        self.consume(&Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(&RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(Block {
                statements: Vec::from([
                    body,
                    Stmt::Expression(Expression {
                        expr: Box::new(inc),
                    }),
                ]),
            });
        };

        body = Stmt::While(stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        });

        if let Some(init) = initializer {
            body = Stmt::Block(Block {
                statements: Vec::from([init, body]),
            })
        };

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        let mut has_paren = false;
        if self.check(&LeftParen) {
            self.advance();
            has_paren = true;
        }

        let condition = self.expression()?;

        if has_paren {
            self.consume(&RightParen, "Expect closing ')' after if condition.")?;
        }

        let then_branch = self.statement()?;
        let else_branch = if self.matches(&[Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Stmt::If(stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }))
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        let mut has_paren = false;
        if self.check(&LeftParen) {
            self.advance();
            has_paren = true;
        }

        let condition = self.expression()?;

        if has_paren {
            self.consume(&RightParen, "Expect closing ')' after while condition.")?;
        }

        let body = self.statement()?;

        Ok(Stmt::While(stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        }))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(&RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let val = self.expression()?;
        self.consume(&Semicolon, "Expected ; after expression")?;
        Ok(Stmt::Expression(Expression {
            expr: Box::new(val),
        }))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.matches(&[Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(var) = expr {
                let name = var.name;
                return Ok(Expr::Assignment(Assignment {
                    name,
                    value: Box::new(value),
                    uuid: uuid_next(),
                }));
            }

            self.error(&equals, "Invalid assignment target.");
            return Err(ParseError {});
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.matches(&[Or, BarBar]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                uuid: uuid_next(),
            })
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.matches(&[And, AmperAmper]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                uuid: uuid_next(),
            })
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison();

        while self.matches(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Ok(Expr::Binary(Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
                uuid: uuid_next(),
            }))
        }

        expr
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term();

        while self.matches(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Ok(Expr::Binary(Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
                uuid: uuid_next(),
            }))
        }

        expr
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor();

        while self.matches(&[Plus, Minus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Ok(Expr::Binary(Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
                uuid: uuid_next(),
            }))
        }

        expr
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary();

        while self.matches(&[Star, Slash, Percentage]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Ok(Expr::Binary(Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
                uuid: uuid_next(),
            }))
        }

        expr
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expr::Unary(Unary {
                operator,
                right: Box::new(right),
                uuid: uuid_next(),
            }))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        loop {
            if self.matches(&[LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut args = Vec::new();

        if !self.check(&RightParen) {
            loop {
                if args.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 arguments.");
                }
                args.push(self.expression()?);
                if !self.matches(&[Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(&RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call(expr::Call {
            callee: Box::new(callee),
            paren,
            arguments: args,
            uuid: uuid_next(),
        }))
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[True]) {
            Ok(Expr::Literal(Literal::new(
                LiteralType::Boolean(true),
                uuid_next(),
            )))
        } else if self.matches(&[False]) {
            Ok(Expr::Literal(Literal::new(
                LiteralType::Boolean(false),
                uuid_next(),
            )))
        } else if self.matches(&[Nil]) {
            Ok(Expr::Literal(Literal::new(LiteralType::Nil, uuid_next())))
        } else if self.matches(&[Number, String]) {
            Ok(Expr::Literal(Literal::new(
                self.previous().literal,
                uuid_next(),
            )))
        } else if self.matches(&[Identifier]) {
            Ok(Expr::Variable(Variable {
                name: self.previous(),
                uuid: uuid_next(),
            }))
        } else if self.matches(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(&RightParen, "Expect ')' after expression.")?;
            Ok(Expr::Grouping(Grouping {
                expr: Box::new(expr),
                uuid: uuid_next(),
            }))
        } else {
            self.error(self.peek(), "Expect expression.");
            Err(ParseError {})
        }
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            self.error(&self.previous(), message);
            Err(ParseError {})
        }
    }

    fn error(&self, token: &Token, message: &str) {
        if token.token_type == Eof {
            crate::error(token.line, &format!(" at end {message}"));
        } else {
            crate::error(token.line, &format!(" at '{}' {message}", token.lexeme));
        }
    }

    fn matches(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().token_type == token_type
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
