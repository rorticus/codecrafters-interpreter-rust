use crate::parser::expr::Expr;

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
}
