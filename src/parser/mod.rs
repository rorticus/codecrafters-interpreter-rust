mod expr;

use crate::{
    lexer::{Token, TokenKind},
    parser::expr::Expr,
};
use std::fmt::Display;

pub enum ParseError {
    UnexpectedToken,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken => write!(f, "Unexpected token"),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        self.pos += 1;
        t
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance().map(|t| &t.kind) {
            Some(TokenKind::True) => Ok(Expr::Literal(expr::LiteralValue::Boolean(true))),
            Some(TokenKind::False) => Ok(Expr::Literal(expr::LiteralValue::Boolean(false))),
            Some(TokenKind::Nil) => Ok(Expr::Literal(expr::LiteralValue::Nil)),
            Some(TokenKind::Number(v)) => Ok(Expr::Literal(expr::LiteralValue::Number(*v))),
            Some(TokenKind::String(v)) => Ok(Expr::Literal(expr::LiteralValue::String(v.clone()))),
            _ => Err(ParseError::UnexpectedToken),
        }
    }
}
