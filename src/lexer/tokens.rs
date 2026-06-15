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
}

pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn literal_str(&self) -> String {
        return "null".to_string();
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
        };

        write!(f, "{name}")
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.kind, self.lexeme, self.literal_str())
    }
}
