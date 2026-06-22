use crate::{lexer::Token, parser::expr::Expr};

#[derive(Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Declaration(Token, Option<Expr>),
    Function {
        name: Token,
        params: Vec<Token>,
        body: Box<Stmt>,
    },
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
    Break(Token),
    Continue(Token),
    Return(Option<Expr>),
}
