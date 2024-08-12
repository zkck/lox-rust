use crate::expr;
use crate::lox;
use crate::object;
use crate::stmt;
use crate::tokens;

#[derive(Debug)]
struct ParseError;

pub struct Parser {
    tokens: Vec<tokens::Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<tokens::Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Vec<stmt::Stmt> {
        let mut statements = vec![];
        while !self.is_at_end() {
            if let Ok(statement) = self.declaration() {
                statements.push(statement)
            }
        }
        return statements;
    }

    fn statement(&mut self) -> Result<stmt::Stmt, ParseError> {
        if self.match_token(tokens::TokenType::Print) {
            return self.print_statement();
        }
        if self.match_token(tokens::TokenType::LeftBrace) {
            return self.block();
        }
        if self.match_token(tokens::TokenType::If) {
            return self.if_statement();
        }
        if self.match_token(tokens::TokenType::While) {
            return self.while_statement();
        }
        if self.match_token(tokens::TokenType::For) {
            return self.for_statement();
        }
        self.expression_statement()
    }

    fn expression(&mut self) -> Result<expr::Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<expr::Expr, ParseError> {
        let expr = self.or()?;
        if self.match_token(tokens::TokenType::Equal) {
            let value = self.assignment()?;
            if let expr::Expr::Variable(name) = expr {
                return Ok(expr::Expr::Assign(name, Box::new(value)));
            }
            self.error("Invalid assignment target.");
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<expr::Expr, ParseError> {
        let mut lhs = self.comparison()?;
        while let Some(operator) = self.match_fn(translate_equality) {
            let rhs = self.comparison()?;
            lhs = expr::Expr::Binary(Box::new(lhs), operator, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn comparison(&mut self) -> Result<expr::Expr, ParseError> {
        let mut lhs = self.term()?;
        while let Some(operator) = self.match_fn(translate_comparison) {
            let rhs = self.term()?;
            lhs = expr::Expr::Binary(Box::new(lhs), operator, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn term(&mut self) -> Result<expr::Expr, ParseError> {
        let mut lhs = self.factor()?;
        while let Some(operator) = self.match_fn(translate_term) {
            let rhs = self.factor()?;
            lhs = expr::Expr::Binary(Box::new(lhs), operator, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn factor(&mut self) -> Result<expr::Expr, ParseError> {
        let mut acc = self.unary()?;
        while let Some(operator) = self.match_fn(translate_factor) {
            let next = self.unary()?;
            acc = expr::Expr::Binary(Box::new(acc), operator, Box::new(next));
        }
        Ok(acc)
    }

    fn unary(&mut self) -> Result<expr::Expr, ParseError> {
        if let Some(operator) = self.match_fn(translate_unary) {
            Ok(expr::Expr::Unary(operator, Box::new(self.unary()?)))
        } else {
            self.call()
        }
    }

    fn primary(&mut self) -> Result<expr::Expr, ParseError> {
        if let Some(literal) = self.match_fn(translate_literal) {
            return Ok(expr::Expr::Literal(literal));
        }
        if self.match_token(tokens::TokenType::LeftParen) {
            let expression = self.expression()?;
            self.consume(
                tokens::TokenType::RightParen,
                "Expected ')' after expression",
            )?;
            return Ok(expr::Expr::Grouping(Box::new(expression)));
        }
        if let Some(name) = self.match_identifier() {
            return Ok(expr::Expr::Variable(name));
        }
        Err(self.error("Expected expression."))
    }

    fn advance(&mut self) -> &tokens::Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn current(&self) -> &tokens::Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &tokens::Token {
        &self.tokens[self.current - 1]
    }

    fn consume(
        &mut self,
        token_type: tokens::TokenType,
        error_message: &str,
    ) -> Result<&tokens::Token, ParseError> {
        if self.current().token_type == token_type {
            Ok(self.advance())
        } else {
            Err(self.error(error_message))
        }
    }

    fn error(&self, message: &str) -> ParseError {
        lox::error_from_token(self.current(), message);
        ParseError {}
    }

    fn synchronize(&mut self) {
        self.advance(); // consume problematic token

        while !self.is_at_end() {
            if self.previous().token_type == tokens::TokenType::Semicolon {
                return;
            }

            match self.current().token_type {
                tokens::TokenType::Class
                | tokens::TokenType::Fun
                | tokens::TokenType::Var
                | tokens::TokenType::For
                | tokens::TokenType::If
                | tokens::TokenType::While
                | tokens::TokenType::Print
                | tokens::TokenType::Return => return,
                _ => self.advance(),
            };
        }
    }

    fn is_at_end(&self) -> bool {
        self.current().token_type == tokens::TokenType::EOF
    }

    fn expression_statement(&mut self) -> Result<stmt::Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(
            tokens::TokenType::Semicolon,
            "Expected ';' after expression",
        )?;
        Ok(stmt::Stmt::Expression(value))
    }

    fn print_statement(&mut self) -> Result<stmt::Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(
            tokens::TokenType::Semicolon,
            "Expected ';' after expression",
        )?;
        Ok(stmt::Stmt::Print(value))
    }

    fn declaration(&mut self) -> Result<stmt::Stmt, ParseError> {
        let maybe_declaration = if self.match_token(tokens::TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        };
        if maybe_declaration.is_err() {
            self.synchronize()
        }
        maybe_declaration
    }

    fn var_declaration(&mut self) -> Result<stmt::Stmt, ParseError> {
        let name = self
            .match_identifier()
            .ok_or_else(|| self.error("Expect variable name."))?;
        let initializer = if self.match_token(tokens::TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            tokens::TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(stmt::Stmt::Var { name, initializer })
    }

    fn block(&mut self) -> Result<stmt::Stmt, ParseError> {
        let mut statements = vec![];
        while self.current().token_type != tokens::TokenType::RightBrace && !self.is_at_end() {
            statements.push(self.declaration()?)
        }
        self.consume(tokens::TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(stmt::Stmt::Block(statements))
    }

    fn if_statement(&mut self) -> Result<stmt::Stmt, ParseError> {
        self.consume(tokens::TokenType::LeftParen, "Expect '(' after if.")?;
        let condition = self.expression()?;
        self.consume(tokens::TokenType::RightParen, "Expect ')' after if.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_token(tokens::TokenType::Else) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(stmt::Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    fn or(&mut self) -> Result<expr::Expr, ParseError> {
        let mut expr = self.and()?;
        while self.match_token(tokens::TokenType::Or) {
            expr = expr::Expr::Logical(
                Box::new(expr),
                expr::LogicalOperator::Or,
                Box::new(self.and()?),
            )
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<expr::Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.match_token(tokens::TokenType::And) {
            expr = expr::Expr::Logical(
                Box::new(expr),
                expr::LogicalOperator::And,
                Box::new(self.equality()?),
            )
        }
        Ok(expr)
    }

    fn while_statement(&mut self) -> Result<stmt::Stmt, ParseError> {
        self.consume(tokens::TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(tokens::TokenType::RightParen, "Expect ')' after 'while'.")?;
        let body = self.statement()?;
        Ok(stmt::Stmt::While(condition, Box::new(body)))
    }

    fn for_statement(&mut self) -> Result<stmt::Stmt, ParseError> {
        self.consume(tokens::TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_token(tokens::TokenType::Semicolon) {
            None
        } else if self.match_token(tokens::TokenType::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.current().token_type == tokens::TokenType::Semicolon {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(tokens::TokenType::Semicolon, "Expect ';' after condition.")?;

        let increment = if self.current().token_type == tokens::TokenType::RightParen {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(tokens::TokenType::RightParen, "Expect ')' after increment.")?;

        let mut body = self.statement()?;

        if let Some(expression) = increment {
            body = stmt::Stmt::Block(vec![body, stmt::Stmt::Expression(expression)])
        }

        body = stmt::Stmt::While(
            condition.unwrap_or(expr::Expr::Literal(object::LoxObject::True)),
            Box::new(body),
        );

        if let Some(statement) = initializer {
            body = stmt::Stmt::Block(vec![statement, body]);
        }

        Ok(body)
    }

    fn match_token(&mut self, token_type: tokens::TokenType) -> bool {
        let is_match = self.current().token_type == token_type;
        if is_match {
            self.advance();
        }
        return is_match;
    }

    fn match_identifier(&mut self) -> Option<String> {
        if let tokens::TokenType::Identifier(s) = &self.current().token_type {
            let some_string = Some(s.to_string());
            self.advance();
            some_string
        } else {
            None
        }
    }

    fn match_fn<T, F>(&mut self, translate: F) -> Option<T>
    where
        F: Fn(&tokens::TokenType) -> Option<T>,
    {
        let translated = translate(&self.current().token_type);
        if translated.is_some() {
            self.advance();
        }
        return translated;
    }

    fn call(&mut self) -> Result<expr::Expr, ParseError> {
        let mut expression = self.primary()?;
        loop {
            if self.match_token(tokens::TokenType::LeftParen) {
                expression = self.complete_call(expression)?;
            } else {
                break;
            }
        }
        Ok(expression)
    }

    /// Given a callee, parses the comma-separated arguments.
    ///
    /// The left parenthesis has already been parsed at this point, and this function will consume
    /// the right parenthesis.
    fn complete_call(&mut self, callee: expr::Expr) -> Result<expr::Expr, ParseError> {
        let mut arguments: Vec<expr::Expr> = vec![];
        if self.current().token_type != tokens::TokenType::RightParen {
            loop {
                if arguments.len() >= 255 {
                    return Err(self.error("Can't have more than 255 arguments."))
                }
                arguments.push(self.expression()?);
                if !self.match_token(tokens::TokenType::Comma) {
                    break
                }
            }
        }
        self.consume(tokens::TokenType::RightParen, "Expect ')' after arguments")?;
        Ok(expr::Expr::Call { callee: Box::new(callee), arguments })
    }
}

fn translate_comparison(token: &tokens::TokenType) -> Option<expr::BinaryOperator> {
    match token {
        tokens::TokenType::Greater => Some(expr::BinaryOperator::GreaterThan),
        tokens::TokenType::GreaterEqual => Some(expr::BinaryOperator::GreaterEqualThan),
        tokens::TokenType::Less => Some(expr::BinaryOperator::LessThan),
        tokens::TokenType::LessEqual => Some(expr::BinaryOperator::LessEqualThan),
        _ => None,
    }
}
fn translate_equality(token: &tokens::TokenType) -> Option<expr::BinaryOperator> {
    match token {
        tokens::TokenType::BangEqual => Some(expr::BinaryOperator::BangEqual),
        tokens::TokenType::EqualEqual => Some(expr::BinaryOperator::EqualEqual),
        _ => None,
    }
}

fn translate_literal(token: &tokens::TokenType) -> Option<object::LoxObject> {
    match token {
        tokens::TokenType::False => Some(object::LoxObject::False),
        tokens::TokenType::True => Some(object::LoxObject::True),
        tokens::TokenType::Nil => Some(object::LoxObject::Nil),
        tokens::TokenType::Number(n) => Some(object::LoxObject::Number(*n)),
        tokens::TokenType::String(s) => Some(object::LoxObject::String(s.to_owned())),
        _ => None,
    }
}

fn translate_unary(token: &tokens::TokenType) -> Option<expr::UnaryOperator> {
    match token {
        tokens::TokenType::Bang => Some(expr::UnaryOperator::Bang),
        tokens::TokenType::Minus => Some(expr::UnaryOperator::Neg),
        _ => None,
    }
}

fn translate_term(token: &tokens::TokenType) -> Option<expr::BinaryOperator> {
    match token {
        tokens::TokenType::Minus => Some(expr::BinaryOperator::Sub),
        tokens::TokenType::Plus => Some(expr::BinaryOperator::Add),
        _ => None,
    }
}

fn translate_factor(token: &tokens::TokenType) -> Option<expr::BinaryOperator> {
    match token {
        tokens::TokenType::Star => Some(expr::BinaryOperator::Mul),
        tokens::TokenType::Slash => Some(expr::BinaryOperator::Div),
        _ => None,
    }
}
