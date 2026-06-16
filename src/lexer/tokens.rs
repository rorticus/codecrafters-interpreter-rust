use std::fmt;

pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Star,
    Plus,
    Minus,
    Semicolon,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Slash,
    String(String),
    Number(f64),
    Identifier(String),
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{:.1}", n) // 42.0
    } else {
        format!("{}", n) // 3.14
    }
}

impl Token {
    pub fn literal_str(&self) -> String {
        match &self.kind {
            TokenKind::String(v) => v.clone(),
            TokenKind::Number(v) => format_number(*v),
            _ => "null".to_string(),
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            TokenKind::LeftParen => "LEFT_PAREN",
            TokenKind::RightParen => "RIGHT_PAREN",
            TokenKind::LeftBrace => "LEFT_BRACE",
            TokenKind::RightBrace => "RIGHT_BRACE",
            TokenKind::Comma => "COMMA",
            TokenKind::Dot => "DOT",
            TokenKind::Star => "STAR",
            TokenKind::Plus => "PLUS",
            TokenKind::Minus => "MINUS",
            TokenKind::Semicolon => "SEMICOLON",
            TokenKind::Equal => "EQUAL",
            TokenKind::EqualEqual => "EQUAL_EQUAL",
            TokenKind::Bang => "BANG",
            TokenKind::BangEqual => "BANG_EQUAL",
            TokenKind::Less => "LESS",
            TokenKind::LessEqual => "LESS_EQUAL",
            TokenKind::Greater => "GREATER",
            TokenKind::GreaterEqual => "GREATER_EQUAL",
            TokenKind::Slash => "SLASH",
            TokenKind::String(_) => "STRING",
            TokenKind::Number(_) => "NUMBER",
            TokenKind::Identifier(_) => "IDENTIFIER",
            TokenKind::And => "AND",
            TokenKind::Class => "CLASS",
            TokenKind::Else => "ELSE",
            TokenKind::False => "FALSE",
            TokenKind::For => "FOR",
            TokenKind::Fun => "FUN",
            TokenKind::If => "IF",
            TokenKind::Nil => "NIL",
            TokenKind::Or => "OR",
            TokenKind::Print => "PRINT",
            TokenKind::Return => "RETURN",
            TokenKind::Super => "SUPER",
            TokenKind::This => "THIS",
            TokenKind::True => "TRUE",
            TokenKind::Var => "VAR",
            TokenKind::While => "WHILE",
        };

        write!(f, "{name}")
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.kind, self.lexeme, self.literal_str())
    }
}
