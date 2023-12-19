pub mod scan;

use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub struct StringValue {
    bytes: Vec<u8>,
}

impl StringValue {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Display for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c\"")?;
        for &byte in self.bytes() {
            if byte != b'"' && (byte == b' ' || byte.is_ascii_graphic()) {
                write!(f, "{}", byte as char)?;
            }
            else {
                write!(f, "\\{byte:02X}")?;
            }
        }
        write!(f, "\"")
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    Identifier(String),
    Integer(u64),
    Boolean(bool),
    NullPointer,
    String(StringValue),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(name) => write!(f, "{name}"),
            Self::Integer(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::NullPointer => write!(f, "null"),
            Self::String(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Plus,
    PlusEqual,
    Plus2,
    Minus,
    MinusEqual,
    Minus2,
    Star,
    StarEqual,
    Slash,
    SlashEqual,
    Percent,
    PercentEqual,
    Ampersand,
    AmpersandEqual,
    Ampersand2,
    Pipe,
    PipeEqual,
    Pipe2,
    Caret,
    CaretEqual,
    Tilde,
    Bang,
    BangEqual,
    Equal,
    Equal2,
    Dot,
    Dot2,
    Comma,
    Colon,
    Semicolon,
    Question,
    AtSign,
    Hash,
    Dollar,
    Backslash,
    ParenLeft,
    ParenRight,
    SquareLeft,
    SquareRight,
    CurlyLeft,
    CurlyRight,
    AngleLeft,
    AngleLeftEqual,
    AngleLeft2,
    AngleLeft2Equal,
    AngleRight,
    AngleRightEqual,
    AngleRight2,
    AngleRight2Equal,
    RightArrow,
    As,
    SizeOf,
    AlignOf,
    If,
    Else,
    While,
    Break,
    Continue,
    Return,
    Let,
    Function,
    Struct,
    Const,
    Mut,
    Literal(Literal),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::PlusEqual => write!(f, "+="),
            Self::Plus2 => write!(f, "++"),
            Self::Minus => write!(f, "-"),
            Self::MinusEqual => write!(f, "-="),
            Self::Minus2 => write!(f, "--"),
            Self::Star => write!(f, "*"),
            Self::StarEqual => write!(f, "*="),
            Self::Slash => write!(f, "/"),
            Self::SlashEqual => write!(f, "/="),
            Self::Percent => write!(f, "%"),
            Self::PercentEqual => write!(f, "%="),
            Self::Ampersand => write!(f, "&"),
            Self::AmpersandEqual => write!(f, "&="),
            Self::Ampersand2 => write!(f, "&&"),
            Self::Pipe => write!(f, "|"),
            Self::PipeEqual => write!(f, "|="),
            Self::Pipe2 => write!(f, "||"),
            Self::Caret => write!(f, "^"),
            Self::CaretEqual => write!(f, "^="),
            Self::Tilde => write!(f, "~"),
            Self::Bang => write!(f, "!"),
            Self::BangEqual => write!(f, "!="),
            Self::Equal => write!(f, "="),
            Self::Equal2 => write!(f, "=="),
            Self::Dot => write!(f, "."),
            Self::Dot2 => write!(f, ".."),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::Semicolon => write!(f, ";"),
            Self::Question => write!(f, "?"),
            Self::AtSign => write!(f, "@"),
            Self::Hash => write!(f, "#"),
            Self::Dollar => write!(f, "$"),
            Self::Backslash => write!(f, "\\"),
            Self::ParenLeft => write!(f, "("),
            Self::ParenRight => write!(f, ")"),
            Self::SquareLeft => write!(f, "["),
            Self::SquareRight => write!(f, "]"),
            Self::CurlyLeft => write!(f, "{{"),
            Self::CurlyRight => write!(f, "}}"),
            Self::AngleLeft => write!(f, "<"),
            Self::AngleLeftEqual => write!(f, "<="),
            Self::AngleLeft2 => write!(f, "<<"),
            Self::AngleLeft2Equal => write!(f, "<<="),
            Self::AngleRight => write!(f, ">"),
            Self::AngleRightEqual => write!(f, ">="),
            Self::AngleRight2 => write!(f, ">>"),
            Self::AngleRight2Equal => write!(f, ">>="),
            Self::RightArrow => write!(f, "->"),
            Self::As => write!(f, "as"),
            Self::SizeOf => write!(f, "sizeof"),
            Self::AlignOf => write!(f, "alignof"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::While => write!(f, "while"),
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),
            Self::Return => write!(f, "return"),
            Self::Let => write!(f, "let"),
            Self::Function => write!(f, "function"),
            Self::Struct => write!(f, "struct"),
            Self::Const => write!(f, "const"),
            Self::Mut => write!(f, "mut"),
            Self::Literal(literal) => write!(f, "{literal}"),
        }
    }
}

pub const SYMBOLIC_TOKENS: &[(&str, Token)] = &[
    ("+", Token::Plus),
    ("+=", Token::PlusEqual),
    ("++", Token::Plus2),
    ("-", Token::Minus),
    ("-=", Token::MinusEqual),
    ("--", Token::Minus2),
    ("*", Token::Star),
    ("*=", Token::StarEqual),
    ("/", Token::Slash),
    ("/=", Token::SlashEqual),
    ("%", Token::Percent),
    ("%=", Token::PercentEqual),
    ("&", Token::Ampersand),
    ("&=", Token::AmpersandEqual),
    ("&&", Token::Ampersand2),
    ("|", Token::Pipe),
    ("|=", Token::PipeEqual),
    ("||", Token::Pipe2),
    ("^", Token::Caret),
    ("^=", Token::CaretEqual),
    ("~", Token::Tilde),
    ("!", Token::Bang),
    ("!=", Token::BangEqual),
    ("=", Token::Equal),
    ("==", Token::Equal2),
    (".", Token::Dot),
    ("..", Token::Dot2),
    (",", Token::Comma),
    (":", Token::Colon),
    (";", Token::Semicolon),
    ("?", Token::Question),
    ("@", Token::AtSign),
    ("#", Token::Hash),
    ("$", Token::Dollar),
    ("\\", Token::Backslash),
    ("(", Token::ParenLeft),
    (")", Token::ParenRight),
    ("[", Token::SquareLeft),
    ("]", Token::SquareRight),
    ("{", Token::CurlyLeft),
    ("}", Token::CurlyRight),
    ("<", Token::AngleLeft),
    ("<=", Token::AngleLeftEqual),
    ("<<", Token::AngleLeft2),
    ("<<=", Token::AngleLeft2Equal),
    (">", Token::AngleRight),
    (">=", Token::AngleRightEqual),
    (">>", Token::AngleRight2),
    (">>=", Token::AngleRight2Equal),
    ("->", Token::RightArrow),
];

pub const KEYWORD_TOKENS: &[(&str, Token)] = &[
    ("as", Token::As),
    ("sizeof", Token::SizeOf),
    ("alignof", Token::AlignOf),
    ("if", Token::If),
    ("else", Token::Else),
    ("while", Token::While),
    ("break", Token::Break),
    ("continue", Token::Continue),
    ("return", Token::Return),
    ("let", Token::Let),
    ("function", Token::Function),
    ("struct", Token::Struct),
    ("const", Token::Const),
    ("mut", Token::Mut),
];

pub fn get_symbolic_token_partial_matches(start_content: &str) -> Vec<&'static Token> {
    SYMBOLIC_TOKENS.iter()
        .filter(|(literal, _)| literal.starts_with(start_content))
        .map(|(_, symbolic_token)| symbolic_token)
        .collect()
}

pub fn get_symbolic_token_match(content: &str) -> Option<&'static Token> {
    SYMBOLIC_TOKENS.iter()
        .find(|(literal, _)| &content == literal)
        .map(|(_, symbolic_token)| symbolic_token)
}

pub fn get_keyword_token_match(content: &str) -> Option<&'static Token> {
    KEYWORD_TOKENS.iter()
        .find(|(keyword, _)| &content == keyword)
        .map(|(_, keyword_token)| keyword_token)
}
