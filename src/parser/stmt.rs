use crate::parser::expr::Expr;

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Declaration(String, Option<Expr>),
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
}
