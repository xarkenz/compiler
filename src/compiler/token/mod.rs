pub mod scan;

use std::fmt;

#[derive(Clone, Debug)]
pub enum Literal {
    Integer(u64),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Token {
    Unknown,
    Plus,
    Minus,
    Star,
    Slash,
    Literal(Literal),
}

pub const SYMBOLIC_TOKENS: &'static [(&'static str, Token)] = &[
    ("+", Token::Plus),
    ("-", Token::Minus),
    ("*", Token::Star),
    ("/", Token::Slash),
];

pub fn get_symbolic_token_matches(start_content: &str) -> Vec<&'static Token> {
    SYMBOLIC_TOKENS.iter()
        .filter_map(|(literal, token)| {
            if literal.starts_with(start_content) {
                Some(token)
            } else {
                None
            }
        })
        .collect()
}
