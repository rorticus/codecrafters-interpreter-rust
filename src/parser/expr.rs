pub enum LiteralValue {
    Nil,
    Boolean(bool),
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
            },
        }
    }
}
