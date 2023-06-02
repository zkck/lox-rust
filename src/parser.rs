use crate::expr;
use crate::lox;
use crate::object;
use crate::stmt;
use crate::tokens;

#[derive(Debug)]
pub struct ParseError;

pub struct Parser {
    tokens: Vec<tokens::Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<tokens::Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Result<Vec<stmt::Stmt>, ParseError> {
        let mut statements = vec![];
        while !self.is_at_end() {
            if let Some(statement) = self.declaration() {
                statements.push(statement)
            }
        }
        return Ok(statements);
    }

    fn statement(&mut self) -> Result<stmt::Stmt, ParseError> {
        match self.current().token_type {
            tokens::TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn expression(&mut self) -> Result<expr::Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<expr::Expr, ParseError> {
        let mut lhs = self.comparison()?;
        while let Some(operator) = match self.current().token_type {
            tokens::TokenType::BangEqual => Some(expr::BinaryOperator::BangEqual),
            tokens::TokenType::EqualEqual => Some(expr::BinaryOperator::EqualEqual),
            _ => None,
        } {
            self.advance();
            let rhs = self.comparison()?;
            lhs = expr::Expr::Binary(Box::new(lhs), operator, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn comparison(&mut self) -> Result<expr::Expr, ParseError> {
        let mut acc = self.term()?;
        while let Some(operator) = match self.current().token_type {
            tokens::TokenType::Greater => Some(expr::BinaryOperator::GreaterThan),
            tokens::TokenType::GreaterEqual => Some(expr::BinaryOperator::GreaterEqualThan),
            tokens::TokenType::Less => Some(expr::BinaryOperator::LessThan),
            tokens::TokenType::LessEqual => Some(expr::BinaryOperator::LessEqualThan),
            _ => None,
        } {
            self.advance(); // consume operator
            let next = self.term()?;
            acc = expr::Expr::Binary(Box::new(acc), operator, Box::new(next));
        }
        Ok(acc)
    }

    fn term(&mut self) -> Result<expr::Expr, ParseError> {
        let mut acc = self.factor()?;
        while let Some(operator) = match self.current().token_type {
            tokens::TokenType::Minus => Some(expr::BinaryOperator::Sub),
            tokens::TokenType::Plus => Some(expr::BinaryOperator::Add),
            _ => None,
        } {
            self.advance();
            let next = self.factor()?;
            acc = expr::Expr::Binary(Box::new(acc), operator, Box::new(next));
        }
        Ok(acc)
    }

    fn factor(&mut self) -> Result<expr::Expr, ParseError> {
        let mut acc = self.unary()?;
        while let Some(operator) = match self.current().token_type {
            tokens::TokenType::Star => Some(expr::BinaryOperator::Mul),
            tokens::TokenType::Slash => Some(expr::BinaryOperator::Div),
            _ => None,
        } {
            self.advance(); // comsume operator
            let next = self.unary()?;
            acc = expr::Expr::Binary(Box::new(acc), operator, Box::new(next));
        }
        Ok(acc)
    }

    fn unary(&mut self) -> Result<expr::Expr, ParseError> {
        if let Some(operator) = match self.current().token_type {
            tokens::TokenType::Bang => Some(expr::UnaryOperator::Bang),
            tokens::TokenType::Minus => Some(expr::UnaryOperator::Neg),
            _ => None,
        } {
            self.advance(); // consume operator
            Ok(expr::Expr::Unary(operator, Box::new(self.unary()?)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<expr::Expr, ParseError> {
        if let Some(literal) = match &self.current().token_type {
            tokens::TokenType::False => Some(object::LoxObject::False),
            tokens::TokenType::True => Some(object::LoxObject::True),
            tokens::TokenType::Nil => Some(object::LoxObject::Nil),
            tokens::TokenType::Number(n) => Some(object::LoxObject::Number(*n)),
            tokens::TokenType::String(s) => Some(object::LoxObject::String(s.to_owned())),
            _ => None,
        } {
            self.advance(); // consume literal
            Ok(expr::Expr::Literal(literal))
        } else if self.current().token_type == tokens::TokenType::LeftParen {
            self.advance(); // consume paren
            let expression = self.expression()?;
            self.consume(
                tokens::TokenType::RightParen,
                "Expected ')' after expression",
            )?;
            Ok(expr::Expr::Grouping(Box::new(expression)))
        } else if self.current().token_type == tokens::TokenType::Identifier {
            let name = self.current().lexeme.clone();
            self.advance();
            Ok(expr::Expr::Variable(name))
        } else {
            Err(Self::error(self.current(), "Expected expression."))
        }
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
            Err(Self::error(self.current(), error_message))
        }
    }

    fn error(token: &tokens::Token, message: &str) -> ParseError {
        lox::error_from_token(token, message);
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

    fn declaration(&mut self) -> Option<stmt::Stmt> {
        let maybe_declaration = if self.current().token_type == tokens::TokenType::Var {
            self.advance();
            self.var_declaration()
        } else {
            self.statement()
        };
        if maybe_declaration.is_err() {
            self.synchronize()
        }
        maybe_declaration.ok()
    }

    fn var_declaration(&mut self) -> Result<stmt::Stmt, ParseError> {
        let name = self
            .consume(tokens::TokenType::Identifier, "Expect variable name")?
            .lexeme
            .clone();
        let initializer = if self.current().token_type == tokens::TokenType::Equal {
            self.advance();
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
}
