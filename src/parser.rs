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
        match self.current().token_type {
            tokens::TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            tokens::TokenType::LeftBrace => {
                self.advance();
                self.block()
            }
            tokens::TokenType::If => {
                self.advance();
                self.if_statement()
            }
            tokens::TokenType::While => {
                self.advance();
                self.while_statement()
            }
            tokens::TokenType::For => {
                self.advance();
                self.for_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn expression(&mut self) -> Result<expr::Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<expr::Expr, ParseError> {
        let expr = self.or()?;
        if self.current().token_type == tokens::TokenType::Equal {
            let equals = self.current().clone();
            self.advance();
            let value = self.assignment()?;
            if let expr::Expr::Variable(name) = expr {
                return Ok(expr::Expr::Assign(name, Box::new(value)));
            }

            Self::error(&equals, "Invalid assignment target.");
        }
        Ok(expr)
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

    fn declaration(&mut self) -> Result<stmt::Stmt, ParseError> {
        let maybe_declaration = if self.current().token_type == tokens::TokenType::Var {
            self.advance();
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
        let else_branch = if self.current().token_type == tokens::TokenType::Else {
            self.advance();
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
        while self.current().token_type == tokens::TokenType::Or {
            self.advance();
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
        while self.current().token_type == tokens::TokenType::And {
            self.advance();
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

        let initializer = if self.matches(tokens::TokenType::Semicolon) {
            None
        } else if self.matches(tokens::TokenType::Var) {
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

    fn matches(&mut self, token_type: tokens::TokenType) -> bool {
        let is_match = self.current().token_type == token_type;
        if is_match {
            self.advance();
        }
        return is_match;
    }
}
