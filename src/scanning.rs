use std::str::CharIndices;

use crate::lox;
use crate::tokens;
use crate::tokens::TokenType;

pub struct Scanner<'s> {
    source: &'s str,
    iter: prepeek::Prepeek<CharIndices<'s>, 2>,
    tokens: Vec<crate::tokens::Token<'s>>,
    start: usize,
    line: usize,
}

impl TokenType<'_> {
    fn from_identifier(identifier: &str) -> TokenType {
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
            s => TokenType::Identifier(s),
        }
    }
}

impl<'s> Scanner<'s> {
    pub fn new(source: &'s str) -> Scanner<'s> {
        Scanner {
            source,
            iter: prepeek::Prepeek::new(source.char_indices()),
            tokens: vec![],
            start: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Vec<tokens::Token<'s>> {
        while let Some((start, _)) = self.iter.peek() {
            self.start = *start;
            self.scan_token();
        }

        self.tokens
            .push(tokens::Token::new(tokens::TokenType::EOF, "", self.line));

        self.tokens
    }

    fn add_token(&mut self, token_type: tokens::TokenType<'s>) {
        self.tokens.push(tokens::Token {
            token_type,
            lexeme: self.current_text(),
            line: self.line,
        })
    }

    fn scan_token(&mut self) {
        let Some((_, startchar)) = self.iter.next() else {
            return;
        };
        match startchar {
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
                    self.advance_while(|c| c != '\n')
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

    fn current_text(&self) -> &'s str {
        match self.iter.peek() {
            Some((current, _)) => &self.source[self.start..*current],
            None => &self.source[self.start..],
        }
    }

    fn identifier(&mut self) {
        self.advance_while(|c| c.is_alphanumeric());
        let identifier = self.current_text();
        self.add_token(TokenType::from_identifier(identifier));
    }

    fn number(&mut self) {
        // consume consecutive digits
        self.advance_while(|c| c.is_digit(10));
        // consume decimal part
        let c1 = self.iter.peek().cloned();
        let c2 = self.iter.peek_nth(1).cloned();
        match (c1, c2) {
            (Some((_, '.')), Some((_, c))) if c.is_digit(10) => {
                // consume '.' and following digits
                self.advance();
                self.advance_while(|c| c.is_digit(10));
            }
            _ => {}
        }
        self.add_token(TokenType::Number(self.current_text().parse().unwrap()))
    }

    fn advance_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some((_, c)) = self.iter.peek() {
            if !predicate(*c) {
                break;
            }
            if *c == '\n' {
                self.line += 1
            }
            self.iter.next();
        }
    }

    fn string(&mut self) {
        self.advance_while(|c| c != '"');
        match self.iter.next() {
            Some((current, '"')) => {
                let value = &self.source[self.start + 1..current];
                self.add_token(TokenType::String(value));
            }
            _ => lox::error(self.line, "Unterminated string."),
        }
    }

    fn advance(&mut self) {
        self.iter.next();
    }

    fn current_matches(&mut self, expected: char) -> bool {
        match self.iter.peek() {
            Some((_, c)) if *c == expected => {
                self.advance();
                true
            }
            _ => false,
        }
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
            Token::new(TokenType::LeftBrace, "{", 1),
            Token::new(TokenType::RightBrace, "}", 1),
            Token::new(TokenType::EOF, "", 1),
        ];
        assert_eq!(scanner.scan_tokens(), expected)
    }

    #[test]
    fn can_parse_string() {
        let scanner = Scanner::new("\"this is a string\"");
        let expected = vec![
            Token::new(
                TokenType::String("this is a string"),
                "\"this is a string\"",
                1,
            ),
            Token::new(TokenType::EOF, "", 1),
        ];
        assert_eq!(scanner.scan_tokens(), expected)
    }

    #[test]
    fn can_parse_number() {
        let scanner = Scanner::new("123.456");
        let expected = vec![
            Token::new(TokenType::Number(123.456), "123.456", 1),
            Token::new(TokenType::EOF, "", 1),
        ];
        assert_eq!(scanner.scan_tokens(), expected)
    }

    #[test]
    fn lines_are_tracked() {
        let scanner = Scanner::new("\n\n()");
        let expected = vec![
            Token::new(TokenType::LeftParen, "(", 3),
            Token::new(TokenType::RightParen, ")", 3),
            Token::new(TokenType::EOF, "", 3),
        ];
        assert_eq!(scanner.scan_tokens(), expected)
    }
}
