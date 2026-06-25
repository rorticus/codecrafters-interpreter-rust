pub mod expr;
pub mod stmt;

use crate::{
    lexer::{Token, TokenKind},
    parser::{
        expr::{Expr, ExprKind},
        stmt::Stmt,
    },
};
use std::fmt::Display;

pub enum ParseError {
    ExpectedToken(TokenKind),
    ExpectedExpr(Token),
    UnexpectedEndOfInput,
    ExpectedIdentifier,
    ExpectedBlock,
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
            ParseError::ExpectedBlock => write!(f, "Expected block"),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    next_id: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            next_id: 0,
        }
    }

    fn make_expr(&mut self, kind: ExprKind, line: usize) -> Expr {
        let id = self.next_id;
        self.next_id += 1;
        Expr { id, kind, line }
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

    fn expect_identifier(&mut self) -> Result<Token, ParseError> {
        let name = match self.peek().map(|k| &k.kind) {
            Some(TokenKind::Identifier(_)) => self.peek().unwrap().clone(),
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
        } else if matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Fun)) {
            self.advance();

            self.parse_function()
        } else {
            self.statement()
        }
    }

    fn parse_function(&mut self) -> Result<Stmt, ParseError> {
        let fn_name = self.expect_identifier()?;

        self.expect(TokenKind::LeftParen)?;

        let mut params = vec![];

        if !matches!(self.peek().map(|t| &t.kind), Some(TokenKind::RightParen)) {
            // parameters
            loop {
                params.push(self.expect_identifier()?);

                if !matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Comma)) {
                    break;
                }

                self.advance();
            }
        }

        self.expect(TokenKind::RightParen)?;

        let body = self.expect_block_statement()?;

        Ok(Stmt::Function {
            name: fn_name,
            params,
            body: Box::new(body),
        })
    }

    fn non_decl_statement(&mut self) -> Result<Stmt, ParseError> {
        let stmt = self.block()?;

        match stmt {
            Stmt::Declaration(name, _) => Err(ParseError::ExpectedExpr(name)),
            _ => Ok(stmt),
        }
    }

    fn expect_block_statement(&mut self) -> Result<Stmt, ParseError> {
        let stmt = self.block()?;

        match stmt {
            Stmt::Block(stmts) => Ok(Stmt::Block(stmts)),
            _ => Err(ParseError::ExpectedBlock),
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
            Some(TokenKind::Break) => {
                let t = self.advance().unwrap().clone();
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Break(t.clone()))
            }
            Some(TokenKind::Continue) => {
                let t = self.advance().unwrap().clone();
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Continue(t.clone()))
            }
            Some(TokenKind::Return) => {
                self.advance();

                let mut return_value = None;

                if !matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Semicolon)) {
                    return_value = Some(self.expression()?);
                }

                self.expect(TokenKind::Semicolon)?;

                Ok(Stmt::Return(return_value))
            }
            Some(TokenKind::Class) => self.parse_class_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Expression(value))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Equal)) {
            let line = self.advance().unwrap().line;

            let value = self.assignment()?;

            match &expr.kind {
                ExprKind::Identifier(name) => {
                    return Ok(self.make_expr(
                        ExprKind::Assign {
                            name: name.clone(),
                            value: Box::new(value),
                        },
                        line,
                    ));
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
            let line = operator.line;
            let right = self.and()?;
            left = self.make_expr(
                ExprKind::Logical {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                line,
            );
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.equality()?;
        while matches!(self.peek().map(|t| &t.kind), Some(TokenKind::And)) {
            let operator = self.advance().unwrap().clone();
            let line = operator.line;
            let right = self.equality()?;
            left = self.make_expr(
                ExprKind::Logical {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                line,
            );
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
            let line = operator.line;

            let right = self.comparison()?;

            left = self.make_expr(
                ExprKind::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                line,
            )
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
            let line = operator.line;

            let right = self.addition()?;

            left = self.make_expr(
                ExprKind::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                line,
            )
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
            let line = operator.line;

            let right = self.multiplication()?;

            left = self.make_expr(
                ExprKind::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                line,
            )
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
            let line = operator.line;

            let right = self.unary()?;

            left = self.make_expr(
                ExprKind::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                line,
            );
        }

        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Bang) | Some(TokenKind::Minus)
        ) {
            let operator = self.advance().unwrap().clone();
            let line = operator.line;
            let right = self.unary()?;
            Ok(self.make_expr(
                ExprKind::Unary {
                    operator,
                    right: Box::new(right),
                },
                line,
            ))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        while matches!(self.peek().map(|k| &k.kind), Some(TokenKind::LeftParen)) {
            let line = self.advance().unwrap().line;
            let mut arguments = vec![];

            if !matches!(self.peek().map(|k| &k.kind), Some(TokenKind::RightParen)) {
                loop {
                    arguments.push(self.expression()?);

                    if !matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Comma)) {
                        break;
                    }
                    self.advance();
                }
            }

            self.expect(TokenKind::RightParen)?;

            expr = self.make_expr(
                ExprKind::Call {
                    expr: Box::new(expr),
                    arguments,
                },
                line,
            )
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let line = self.peek().map(|t| t.line).unwrap_or(0);

        match self.advance() {
            Some(t) => {
                match &t.kind {
                    TokenKind::True => {
                        Ok(self
                            .make_expr(ExprKind::Literal(expr::LiteralValue::Boolean(true)), line))
                    }
                    TokenKind::False => {
                        Ok(self
                            .make_expr(ExprKind::Literal(expr::LiteralValue::Boolean(false)), line))
                    }
                    TokenKind::Nil => {
                        Ok(self.make_expr(ExprKind::Literal(expr::LiteralValue::Nil), line))
                    }
                    TokenKind::Number(v) => {
                        let n = v.clone();
                        Ok(self.make_expr(ExprKind::Literal(expr::LiteralValue::Number(n)), line))
                    }
                    TokenKind::String(v) => {
                        let str = v.clone();
                        Ok(
                            self.make_expr(
                                ExprKind::Literal(expr::LiteralValue::String(str)),
                                line,
                            ),
                        )
                    }
                    TokenKind::LeftParen => {
                        let inner = self.expression()?; // recurse all the way back up
                        self.expect(TokenKind::RightParen)?; // consume the closing ')'
                        Ok(self.make_expr(ExprKind::Grouping(Box::new(inner)), line))
                    }
                    TokenKind::Identifier(_) => {
                        let token = t.clone();
                        Ok(self.make_expr(ExprKind::Identifier(token), line))
                    }
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

        let statement = self.non_decl_statement()?;
        let mut else_statement = None;

        if matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Else)) {
            self.advance();

            else_statement = Some(Box::new(self.non_decl_statement()?));
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

        let block = self.non_decl_statement()?;

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
            if matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Var)) {
                initializer = Some(Box::new(self.declaration()?));
            } else {
                initializer = Some(Box::new(self.parse_expression_statement()?));
            }
        } else {
            self.expect(TokenKind::Semicolon)?;
        }

        if !matches!(self.peek().map(|k| &k.kind), Some(TokenKind::Semicolon)) {
            condition = Some(self.expression()?);
        }

        self.expect(TokenKind::Semicolon)?;

        if !matches!(self.peek().map(|k| &k.kind), Some(TokenKind::RightParen)) {
            incrementer = Some(self.expression()?);
        }

        self.expect(TokenKind::RightParen)?;

        let block = self.non_decl_statement()?;

        Ok(Stmt::For {
            initializer,
            condition,
            increment: incrementer,
            block: Box::new(block),
        })
    }

    fn parse_class_statement(&mut self) -> Result<Stmt, ParseError> {
        // consume the class keyword
        self.advance();

        let name = self.expect_identifier()?;

        self.expect(TokenKind::LeftBrace)?;

        let mut stmts = Vec::new();
        while !matches!(self.peek().map(|t| &t.kind), Some(TokenKind::RightBrace)) {
            stmts.push(self.parse_function()?);
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::Class {
            name,
            methods: stmts,
        })
    }
}
