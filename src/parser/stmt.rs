use crate::{lexer::Token, parser::expr::Expr};

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Declaration(Token, Option<Expr>),
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        block: Box<Stmt>,
    },
    For {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        block: Box<Stmt>,
    },
}
