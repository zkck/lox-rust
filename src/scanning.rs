use std::iter::FromIterator;

use crate::lox;
use crate::tokens;
use crate::tokens::TokenType;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<crate::tokens::Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl TokenType {
    fn from_identifier(identifier: &str) -> Self {
        match identifier {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            s => TokenType::Identifier(s.to_string()),
        }
    }
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Vec<tokens::Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(tokens::Token::new(
            tokens::TokenType::EOF,
            String::new(),
            self.line,
        ));

        self.tokens
    }

    fn scan_token(&mut self) {
        match self.advance() {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token = if self.current_matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token)
            }
            '=' => {
                let token = if self.current_matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token)
            }
            '<' => {
                let token = if self.current_matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token)
            }
            '>' => {
                let token = if self.current_matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token)
            }
            '/' => {
                if self.current_matches('/') {
                    // it's a comment
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => self.string(),
            c => {
                if c.is_digit(10) {
                    self.number();
                } else if c.is_alphabetic() {
                    self.identifier();
                } else {
                    lox::error(self.line, "Unexpected character.")
                }
            }
        }
    }

    fn current_text(&self) -> String {
        String::from_iter(&self.source[self.start..self.current])
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }
        self.add_token(TokenType::from_identifier(&self.current_text()))
    }

    fn number(&mut self) {
        // consume consecutive digits
        while self.peek().is_digit(10) {
            self.advance();
        }
        // consume decimal part
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // consume '.'
            self.advance();
            while self.peek().is_digit(10) {
                self.advance();
            }
        }
        self.add_token(TokenType::Number(
            String::from_iter(&self.source[self.start..self.current])
                .parse()
                .unwrap(),
        ))
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current + 1]
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            lox::error(self.line, "Unterminated string.");
            return;
        }
        // consume closing delimiter
        self.advance();
        let value = String::from_iter(&self.source[self.start + 1..self.current - 1]);
        self.add_token(TokenType::String(value))
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn current_matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        // current matches, consume
        self.current += 1;
        true
    }

    fn add_token(&mut self, token_type: tokens::TokenType) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(tokens::Token::new(
            token_type,
            String::from_iter(text),
            self.line,
        ))
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::Token;

    use super::*;

    #[test]
    fn can_parse_braces() {
        let scanner = Scanner::new("{}");
        let expected = vec![
            Token::new(TokenType::LeftBrace, "{".to_string(), 1),
            Token::new(TokenType::RightBrace, "}".to_string(), 1),
            Token::new(TokenType::EOF, "".to_string(), 1),
        ];
        assert_eq!(scanner.scan_tokens(), expected)
    }

    #[test]
    fn can_parse_string() {
        let scanner = Scanner::new("\"this is a string\"");
        let expected = vec![
            Token::new(
                TokenType::String("this is a string".to_string()),
                "\"this is a string\"".to_string(),
                1,
            ),
            Token::new(TokenType::EOF, "".to_string(), 1),
        ];
        assert_eq!(scanner.scan_tokens(), expected)
    }

    #[test]
    fn can_parse_number() {
        let scanner = Scanner::new("123.456");
        let expected = vec![
            Token::new(TokenType::Number(123.456), "123.456".to_string(), 1),
            Token::new(TokenType::EOF, "".to_string(), 1),
        ];
        assert_eq!(scanner.scan_tokens(), expected)
    }

    #[test]
    fn lines_are_tracked() {
        let scanner = Scanner::new("\n\n()");
        let expected = vec![
            Token::new(TokenType::LeftParen, "(".to_string(), 3),
            Token::new(TokenType::RightParen, ")".to_string(), 3),
            Token::new(TokenType::EOF, "".to_string(), 3),
        ];
        assert_eq!(scanner.scan_tokens(), expected)
    }
}
