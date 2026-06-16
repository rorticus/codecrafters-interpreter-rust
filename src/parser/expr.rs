use crate::lexer::tokens::format_number;

pub enum LiteralValue {
    Nil,
    Boolean(bool),
    Number(f64),
}

pub enum Expr {
    Literal(LiteralValue),
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
            },
        }
    }
}
