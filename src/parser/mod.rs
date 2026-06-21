pub mod expr;
pub mod stmt;

use crate::{
    lexer::{Token, TokenKind},
    parser::{expr::Expr, stmt::Stmt},
};
use std::fmt::Display;

pub enum ParseError {
    ExpectedToken(TokenKind),
    ExpectedExpr(Token),
    UnexpectedEndOfInput,
    ExpectedIdentifier,
    InvalidAssignmentTarget,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ExpectedToken(t) => write!(f, "Expected {t}"),
            ParseError::ExpectedExpr(t) => write!(
                f,
                "[Line {}] Error at '{}': Expect expression.",
                t.line, t.lexeme
            ),
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ParseError::ExpectedIdentifier => write!(f, "Expected identifier"),
            ParseError::InvalidAssignmentTarget => write!(f, "Invalid assignment target"),
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

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let name = match self.peek().map(|k| &k.kind) {
            Some(TokenKind::Identifier(name)) => name.clone(),
            _ => return Err(ParseError::ExpectedIdentifier),
        };

        self.advance();
        Ok(name)
    }

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    pub fn parse(&mut self) -> Vec<Result<Stmt, ParseError>> {
        let mut statements = Vec::new();
        while self.peek().is_some() {
            statements.push(self.block());
        }

        statements
    }

    fn block(&mut self) -> Result<Stmt, ParseError> {
        if matches!(self.peek().map(|k| &k.kind), Some(TokenKind::LeftBrace)) {
            self.advance();
            let mut statements = Vec::new();

            while !matches!(self.peek().map(|k| &k.kind), Some(TokenKind::RightBrace)) {
                statements.push(self.block()?);
            }

            self.advance();

            Ok(Stmt::Block(statements))
        } else {
            self.declaration()
        }
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Var)) {
            self.advance();
            let var_name = self.expect_identifier()?;

            if matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Equal)) {
                self.expect(TokenKind::Equal)?;
                let expr = self.expression()?;
                self.expect(TokenKind::Semicolon)?;

                Ok(Stmt::Declaration(var_name, Some(expr)))
            } else {
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Declaration(var_name, None))
            }
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().map(|k| &k.kind) {
            Some(TokenKind::Print) => {
                self.advance();
                let value = self.expression()?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Print(value))
            }
            Some(TokenKind::If) => Ok(self.parse_if()?),
            Some(TokenKind::While) => Ok(self.parse_while()?),
            Some(TokenKind::For) => self.parse_for(),
            _ => {
                let value = self.expression()?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Expression(value))
            }
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Equal)) {
            self.advance();

            let value = self.assignment()?;

            match expr {
                Expr::Identifier(name) => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    });
                }
                _ => return Err(ParseError::InvalidAssignmentTarget),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.and()?;
        while matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Or)) {
            let operator = self.advance().unwrap().clone();
            let right = self.and()?;
            left = Expr::Logical {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.equality()?;
        while matches!(self.peek().map(|t| &t.kind), Some(TokenKind::And)) {
            let operator = self.advance().unwrap().clone();
            let right = self.equality()?;
            left = Expr::Logical {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.comparison()?;

        while matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::EqualEqual) | Some(TokenKind::BangEqual)
        ) {
            let operator = self.advance().unwrap().clone();

            let right = self.comparison()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.addition()?;

        while matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Less)
                | Some(TokenKind::LessEqual)
                | Some(TokenKind::Greater)
                | Some(TokenKind::GreaterEqual)
        ) {
            let operator = self.advance().unwrap().clone();

            let right = self.addition()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        }

        Ok(left)
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
        match self.advance() {
            Some(t) => {
                match &t.kind {
                    TokenKind::True => Ok(Expr::Literal(expr::LiteralValue::Boolean(true))),
                    TokenKind::False => Ok(Expr::Literal(expr::LiteralValue::Boolean(false))),
                    TokenKind::Nil => Ok(Expr::Literal(expr::LiteralValue::Nil)),
                    TokenKind::Number(v) => Ok(Expr::Literal(expr::LiteralValue::Number(*v))),
                    TokenKind::String(v) => {
                        Ok(Expr::Literal(expr::LiteralValue::String(v.clone())))
                    }
                    TokenKind::LeftParen => {
                        let inner = self.expression()?; // recurse all the way back up
                        self.expect(TokenKind::RightParen)?; // consume the closing ')'
                        Ok(Expr::Grouping(Box::new(inner)))
                    }
                    TokenKind::Identifier(_) => Ok(Expr::Identifier(t.clone())),
                    _ => Err(ParseError::ExpectedExpr(t.clone())),
                }
            }
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        self.expect(TokenKind::LeftParen)?;

        let condition = self.expression()?;

        self.expect(TokenKind::RightParen)?;

        let statement = self.block()?;
        let mut else_statement = None;

        if matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Else)) {
            self.advance();

            else_statement = Some(Box::new(self.block()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(statement),
            else_branch: else_statement,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        self.expect(TokenKind::LeftParen)?;
        let condition = self.expression()?;
        self.expect(TokenKind::RightParen)?;

        let block = self.block()?;

        return Ok(Stmt::While {
            condition,
            block: Box::new(block),
        });
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        let mut initializer = None;
        let mut condition = None;
        let mut incrementer = None;

        self.advance();

        self.expect(TokenKind::LeftParen)?;

        if !matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Semicolon)) {
            initializer = Some(Box::new(self.declaration()?));
        }

        if !matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Semicolon)) {
            condition = Some(self.expression()?);
            self.expect(TokenKind::Semicolon)?;
        }

        if !matches!(self.peek().map(|k| &k.kind), Some(TokenKind::RightParen)) {
            incrementer = Some(self.expression()?);
        }

        self.expect(TokenKind::RightParen)?;

        let block = self.block()?;

        Ok(Stmt::For {
            initializer,
            condition,
            increment: incrementer,
            block: Box::new(block),
        })
    }
}
