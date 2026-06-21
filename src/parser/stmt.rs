use crate::parser::expr::Expr;

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Declaration(String, Option<Expr>),
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        thenBranch: Box<Stmt>,
        elseBranch: Option<Box<Stmt>>,
    },
}
