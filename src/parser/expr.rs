use crate::lexer::Token;
use crate::lexer::tokens::format_number;

pub enum LiteralValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

pub enum Expr {
    Literal(LiteralValue),
    Identifier(String),
    Assign {
        name: String,
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
                format!("{}", t)
            }
            Expr::Assign { name, value } => {
                format!("{} = {}", name, value.pretty_print())
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
        }
    }
}
