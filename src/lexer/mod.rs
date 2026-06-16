mod tokens;

use std::fmt::Display;
use tokens::Token;

use crate::lexer::LexError::{UnexpectedCharacter, UnterminatedString};

pub enum LexError {
    UnexpectedCharacter(usize, char),
    UnterminatedString(usize),
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnexpectedCharacter(line, c) => {
                write!(f, "[line {}] Error: Unexpected character: {}", line, c)
            }
            UnterminatedString(line) => {
                write!(f, "[line {}] Error: Unterminated string.", line)
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

    fn slice(&self, start: usize, end: usize) -> String {
        return self.chars[start..end].iter().collect();
    }

    fn keyword(&self, ident: &str) -> Option<tokens::TokenKind> {
        match ident {
            "and" => Some(tokens::TokenKind::And),
            "class" => Some(tokens::TokenKind::Class),
            "else" => Some(tokens::TokenKind::Else),
            "false" => Some(tokens::TokenKind::False),
            "for" => Some(tokens::TokenKind::For),
            "fun" => Some(tokens::TokenKind::Fun),
            "if" => Some(tokens::TokenKind::If),
            "nil" => Some(tokens::TokenKind::Nil),
            "or" => Some(tokens::TokenKind::Or),
            "print" => Some(tokens::TokenKind::Print),
            "return" => Some(tokens::TokenKind::Return),
            "super" => Some(tokens::TokenKind::Super),
            "this" => Some(tokens::TokenKind::This),
            "true" => Some(tokens::TokenKind::True),
            "var" => Some(tokens::TokenKind::Var),
            "while" => Some(tokens::TokenKind::While),
            _ => None,
        }
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

                Some('"') => {
                    let start = self.pos;

                    self.advance();

                    loop {
                        match self.peek() {
                            Some('"') => {
                                self.advance();
                                let end = self.pos;

                                return Some(Ok(Token {
                                    kind: tokens::TokenKind::String(self.slice(start + 1, end - 1)),
                                    lexeme: self.slice(start, end),
                                    line: self.line,
                                }));
                            }
                            Some(_) => {
                                self.advance();
                            }
                            None => return Some(Err(LexError::UnterminatedString(self.line))),
                        }
                    }
                }

                Some(c) if c.is_ascii_digit() => {
                    let start = self.pos;
                    self.advance();

                    loop {
                        match self.peek() {
                            Some(c) if c.is_ascii_digit() || c == '.' => {
                                self.advance();
                            }
                            _ => {
                                let end = self.pos;

                                let number_str = self.slice(start, end);
                                let as_float: f64 = number_str.parse().unwrap();

                                return Some(Ok(Token {
                                    kind: tokens::TokenKind::Number(as_float),
                                    lexeme: number_str,
                                    line: self.line,
                                }));
                            }
                        }
                    }
                }

                Some(c) if c.is_alphabetic() || c == '_' => {
                    let start = self.pos;
                    self.advance();

                    loop {
                        match self.peek() {
                            Some(c) if c.is_alphanumeric() || c == '_' => {
                                self.advance();
                            }
                            _ => {
                                let end = self.pos;

                                let lexeme = self.slice(start, end);

                                return Some(Ok(Token {
                                    kind: self.keyword(&lexeme).unwrap_or(
                                        tokens::TokenKind::Identifier(self.slice(start, end)),
                                    ),
                                    lexeme,
                                    line: self.line,
                                }));
                            }
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
