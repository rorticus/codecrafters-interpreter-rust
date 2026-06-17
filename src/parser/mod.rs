mod expr;

use crate::{
    lexer::{Token, TokenKind},
    parser::expr::Expr,
};
use std::fmt::Display;

pub enum ParseError {
    UnexpectedToken,
    ExpectedToken(TokenKind),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken => write!(f, "Unexpected token"),
            ParseError::ExpectedToken(t) => write!(f, "Expected {t}"),
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

    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        match self.peek().map(|t| &t.kind) {
            Some(k) if k == &kind => {
                self.advance();
                Ok(())
            }
            _ => Err(ParseError::ExpectedToken(kind)),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.addition()
    }

    fn addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.multiplication()?;

        while matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Plus) | Some(TokenKind::Minus)
        ) {
            let operator = self.advance().unwrap().clone();

            let right = self.multiplication()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.unary()?;

        while matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Star) | Some(TokenKind::Slash)
        ) {
            let operator = self.advance().unwrap().clone();

            let right = self.unary()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Bang) | Some(TokenKind::Minus)
        ) {
            let operator = self.advance().unwrap().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance().map(|t| &t.kind) {
            Some(TokenKind::True) => Ok(Expr::Literal(expr::LiteralValue::Boolean(true))),
            Some(TokenKind::False) => Ok(Expr::Literal(expr::LiteralValue::Boolean(false))),
            Some(TokenKind::Nil) => Ok(Expr::Literal(expr::LiteralValue::Nil)),
            Some(TokenKind::Number(v)) => Ok(Expr::Literal(expr::LiteralValue::Number(*v))),
            Some(TokenKind::String(v)) => Ok(Expr::Literal(expr::LiteralValue::String(v.clone()))),
            Some(TokenKind::LeftParen) => {
                let inner = self.expression()?; // recurse all the way back up
                self.expect(TokenKind::RightParen)?; // consume the closing ')'
                Ok(Expr::Grouping(Box::new(inner)))
            }
            _ => Err(ParseError::UnexpectedToken),
        }
    }
}
