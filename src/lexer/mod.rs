mod tokens;

use std::{fmt::Display, format};
use tokens::Token;

use crate::lexer::LexError::UnexpectedCharacter;

pub enum LexError {
    UnexpectedCharacter(char),
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnexpectedCharacter(c) => write!(f, "Unexpected character '{}'", c),
        }
    }
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;

        c
    }
}

impl Iterator for Lexer {
    type Item = Result<Token, LexError>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.peek() {
                None => return None,
                Some('(') => {
                    self.advance();

                    return Some(Ok(Token {
                        kind: tokens::TokenKind::LeftParen,
                        lexeme: "(".to_string(),
                        line: self.line,
                    }));
                }
                Some(')') => {
                    self.advance();

                    return Some(Ok(Token {
                        kind: tokens::TokenKind::RightParen,
                        lexeme: ")".to_string(),
                        line: self.line,
                    }));
                }
                Some('\n') => {
                    self.advance();
                    self.line += 1;
                }
                Some(c) => return Some(Err(LexError::UnexpectedCharacter(c))),
            }
        }
    }
}
