mod tokens;

use std::fmt::Display;
use tokens::Token;

use crate::lexer::LexError::UnexpectedCharacter;

pub enum LexError {
    UnexpectedCharacter(usize, char),
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnexpectedCharacter(line, c) => {
                write!(f, "[line {}] Error: Unexpected character: {}", line, c)
            }
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
                Some('.') | Some('*') | Some(',') | Some('{') | Some('}') | Some('(')
                | Some(')') | Some('+') | Some('-') | Some(';') => {
                    let c = self.advance().unwrap_or_default();

                    return Some(Ok(Token {
                        kind: match c {
                            '.' => tokens::TokenKind::Dot,
                            '*' => tokens::TokenKind::Star,
                            ',' => tokens::TokenKind::Comma,
                            '{' => tokens::TokenKind::LeftBrace,
                            '}' => tokens::TokenKind::RightBrace,
                            '(' => tokens::TokenKind::LeftParen,
                            ')' => tokens::TokenKind::RightParen,
                            '+' => tokens::TokenKind::Plus,
                            '-' => tokens::TokenKind::Minus,
                            ';' => tokens::TokenKind::Semicolon,
                            _ => panic!("shouldn't be here"),
                        },
                        lexeme: c.to_string(),
                        line: self.line,
                    }));
                }

                Some('=') => {
                    self.advance();
                    match self.peek() {
                        Some('=') => {
                            self.advance();
                            return Some(Ok(Token {
                                kind: tokens::TokenKind::EqualEqual,
                                lexeme: "==".to_string(),
                                line: self.line,
                            }));
                        }
                        _ => {
                            return Some(Ok(Token {
                                kind: tokens::TokenKind::Equal,
                                lexeme: "=".to_string(),
                                line: self.line,
                            }));
                        }
                    }
                }

                Some('!') => {
                    self.advance();

                    match self.peek() {
                        Some('=') => {
                            self.advance();

                            return Some(Ok(Token {
                                kind: tokens::TokenKind::BangEqual,
                                lexeme: "!=".to_string(),
                                line: self.line,
                            }));
                        }
                        _ => {
                            return Some(Ok(Token {
                                kind: tokens::TokenKind::Bang,
                                lexeme: "!".to_string(),
                                line: self.line,
                            }));
                        }
                    }
                }

                Some('<') => {
                    self.advance();

                    match self.peek() {
                        Some('=') => {
                            self.advance();

                            return Some(Ok(Token {
                                kind: tokens::TokenKind::LessEqual,
                                lexeme: "<=".to_string(),
                                line: self.line,
                            }));
                        }
                        _ => {
                            return Some(Ok(Token {
                                kind: tokens::TokenKind::Less,
                                lexeme: "<".to_string(),
                                line: self.line,
                            }));
                        }
                    }
                }

                Some('>') => {
                    self.advance();

                    match self.peek() {
                        Some('=') => {
                            self.advance();

                            return Some(Ok(Token {
                                kind: tokens::TokenKind::GreaterEqual,
                                lexeme: ">=".to_string(),
                                line: self.line,
                            }));
                        }
                        _ => {
                            return Some(Ok(Token {
                                kind: tokens::TokenKind::Greater,
                                lexeme: ">".to_string(),
                                line: self.line,
                            }));
                        }
                    }
                }

                Some('/') => {
                    self.advance();

                    match self.peek() {
                        Some('/') => {
                            self.advance();

                            loop {
                                match self.peek() {
                                    Some('\n') => {
                                        break;
                                    }
                                    Some(_) => {
                                        self.advance();
                                    }
                                    None => {
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {
                            return Some(Ok(Token {
                                kind: tokens::TokenKind::Slash,
                                lexeme: "/".to_string(),
                                line: self.line,
                            }));
                        }
                    }
                }

                Some('\t') | Some(' ') => {
                    self.advance();
                }

                Some('\n') => {
                    self.advance();
                    self.line += 1;
                }
                Some(c) => {
                    self.advance();
                    return Some(Err(LexError::UnexpectedCharacter(self.line, c)));
                }
            }
        }
    }
}
