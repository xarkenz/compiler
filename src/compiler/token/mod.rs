pub mod scan;

use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    Identifier(String),
    Integer(u64),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(name) => write!(f, "{name}"),
            Self::Integer(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Plus,
    Minus,
    Star,
    Slash,
    Semicolon,
    Print,
    Literal(Literal),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plus => write!(f, "plus"),
            Self::Minus => write!(f, "minus"),
            Self::Star => write!(f, "star"),
            Self::Slash => write!(f, "slash"),
            Self::Semicolon => write!(f, "semicolon"),
            Self::Print => write!(f, "'print'"),
            Self::Literal(literal) => literal.fmt(f),
        }
    }
}

pub const SYMBOLIC_TOKENS: &[(&str, Token)] = &[
    ("+", Token::Plus),
    ("-", Token::Minus),
    ("*", Token::Star),
    ("/", Token::Slash),
    (";", Token::Semicolon),
];

pub const KEYWORD_TOKENS: &[(&str, Token)] = &[
    ("print", Token::Print),
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

pub fn get_keyword_token(content: &str) -> Option<&'static Token> {
    KEYWORD_TOKENS.iter()
        .find(|(keyword, _)| &content == keyword)
        .map(|(_, keyword_token)| keyword_token)
}
