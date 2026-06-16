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
}

pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn literal_str(&self) -> &str {
        match &self.kind {
            TokenKind::String(v) => v,
            _ => "null",
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
        };

        write!(f, "{name}")
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.kind, self.lexeme, self.literal_str())
    }
}
