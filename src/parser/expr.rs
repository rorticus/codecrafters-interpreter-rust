use crate::lexer::Token;
use crate::lexer::tokens::format_number;

#[derive(Clone)]
pub enum LiteralValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

#[derive(Clone)]
pub enum Expr {
    Literal(LiteralValue),
    Identifier(Token),
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        expr: Box<Expr>,
        arguments: Vec<Expr>,
    },
}

impl Expr {
    pub fn pretty_print(&self) -> String {
        match self {
            Expr::Literal(value) => match value {
                LiteralValue::Nil => "nil".to_string(),
                LiteralValue::Boolean(b) => {
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                LiteralValue::Number(v) => format_number(*v),
                LiteralValue::String(v) => v.clone(),
            },
            Expr::Grouping(e) => {
                format!("(group {})", e.pretty_print())
            }
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.lexeme, right.pretty_print())
            }
            Expr::Identifier(t) => {
                format!("{}", t.lexeme)
            }
            Expr::Assign { name, value } => {
                format!("{} = {}", name, value.pretty_print())
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                format!(
                    "{} {} {}",
                    left.pretty_print(),
                    operator.lexeme,
                    right.pretty_print()
                )
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                format!(
                    "({} {} {})",
                    operator.lexeme,
                    left.pretty_print(),
                    right.pretty_print()
                )
            }
            Expr::Call { expr, arguments } => format!(
                "{}({})",
                expr.pretty_print(),
                arguments
                    .iter()
                    .map(|a| a.pretty_print())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
