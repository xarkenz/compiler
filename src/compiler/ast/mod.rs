pub mod parse;

use crate::token::*;

use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Associativity {
    LeftToRight,
    RightToLeft,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Precedence {
    // Highest
    Postfix,
    Prefix,
    Conversion,
    Multiplicative,
    Additive,
    BitwiseShift,
    Inequality,
    Equality,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
    Conditional,
    Assignment,
    // Lowest
}

impl Precedence {
    pub fn associativity(&self) -> Associativity {
        match self {
            Self::Prefix | Self::Conditional | Self::Assignment => Associativity::RightToLeft,
            _ => Associativity::LeftToRight
        }
    }
}

impl PartialOrd for Precedence {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Compare the internal values, then reverse the ordering because
        // the lowest internal value represents the highest precedence
        (*self as isize).partial_cmp(&(*other as isize))
            .map(std::cmp::Ordering::reverse)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UnaryOperation {
    PostIncrement,
    PostDecrement,
    PreIncrement,
    PreDecrement,
    Positive,
    Negative,
    BitwiseNot,
    LogicalNot,
    Reference,
    Dereference,
    GetSize,
    GetAlign,
}

impl UnaryOperation {
    pub fn precedence(&self) -> Precedence {
        match self {
            Self::PostIncrement | Self::PostDecrement => Precedence::Postfix,
            Self::PreIncrement | Self::PreDecrement | Self::Positive | Self::Negative
                | Self::BitwiseNot | Self::LogicalNot | Self::Reference | Self::Dereference
                | Self::GetSize | Self::GetAlign => Precedence::Prefix,
        }
    }

    pub fn associativity(&self) -> Associativity {
        self.precedence().associativity()
    }

    pub fn from_prefix_token(token: &Token) -> Option<Self> {
        match token {
            Token::Plus => Some(Self::Positive),
            Token::Plus2 => Some(Self::PreIncrement),
            Token::Minus => Some(Self::Negative),
            Token::Minus2 => Some(Self::PreDecrement),
            Token::Star => Some(Self::Dereference),
            Token::Ampersand => Some(Self::Reference),
            Token::Tilde => Some(Self::BitwiseNot),
            Token::Bang => Some(Self::LogicalNot),
            Token::SizeOf => Some(Self::GetSize),
            Token::AlignOf => Some(Self::GetAlign),
            _ => None
        }
    }

    pub fn from_postfix_token(token: &Token) -> Option<Self> {
        match token {
            Token::Plus2 => Some(Self::PostIncrement),
            Token::Minus2 => Some(Self::PostDecrement),
            _ => None
        }
    }

    pub fn notation(&self) -> &'static str {
        match self {
            Self::PostIncrement => "++", // FIXME
            Self::PostDecrement => "--", // FIXME
            Self::PreIncrement => "++",
            Self::PreDecrement => "--",
            Self::Positive => "+",
            Self::Negative => "-",
            Self::BitwiseNot => "~",
            Self::LogicalNot => "!",
            Self::Reference => "&",
            Self::Dereference => "*",
            Self::GetSize => "sizeof ",
            Self::GetAlign => "alignof ",
        }
    }
}

impl fmt::Display for UnaryOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.notation())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BinaryOperation {
    Subscript,
    Access,
    DerefAccess,
    Convert,
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Equal,
    NotEqual,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
    Assign,
    MultiplyAssign,
    DivideAssign,
    RemainderAssign,
    AddAssign,
    SubtractAssign,
    ShiftLeftAssign,
    ShiftRightAssign,
    BitwiseAndAssign,
    BitwiseXorAssign,
    BitwiseOrAssign,
}

impl BinaryOperation {
    pub fn precedence(&self) -> Precedence {
        match self {
            Self::Subscript | Self::Access | Self::DerefAccess => Precedence::Postfix,
            Self::Convert => Precedence::Conversion,
            Self::Multiply | Self::Divide | Self::Remainder => Precedence::Multiplicative,
            Self::Add | Self::Subtract => Precedence::Additive,
            Self::ShiftLeft | Self::ShiftRight => Precedence::BitwiseShift,
            Self::LessThan | Self::LessEqual | Self::GreaterThan | Self::GreaterEqual => Precedence::Inequality,
            Self::Equal | Self::NotEqual => Precedence::Equality,
            Self::BitwiseAnd => Precedence::BitwiseAnd,
            Self::BitwiseXor => Precedence::BitwiseXor,
            Self::BitwiseOr => Precedence::BitwiseOr,
            Self::LogicalAnd => Precedence::LogicalAnd,
            Self::LogicalOr => Precedence::LogicalOr,
            Self::Assign | Self::MultiplyAssign | Self::DivideAssign | Self::RemainderAssign
                | Self::AddAssign | Self::SubtractAssign | Self::ShiftLeftAssign | Self::ShiftRightAssign
                | Self::BitwiseAndAssign | Self::BitwiseXorAssign | Self::BitwiseOrAssign => Precedence::Assignment,
        }
    }

    pub fn associativity(&self) -> Associativity {
        self.precedence().associativity()
    }

    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Plus => Some(Self::Add),
            Token::PlusEqual => Some(Self::AddAssign),
            Token::Minus => Some(Self::Subtract),
            Token::MinusEqual => Some(Self::SubtractAssign),
            Token::Star => Some(Self::Multiply),
            Token::StarEqual => Some(Self::MultiplyAssign),
            Token::Slash => Some(Self::Divide),
            Token::SlashEqual => Some(Self::DivideAssign),
            Token::Percent => Some(Self::Remainder),
            Token::PercentEqual => Some(Self::RemainderAssign),
            Token::Ampersand => Some(Self::BitwiseAnd),
            Token::AmpersandEqual => Some(Self::BitwiseAndAssign),
            Token::Ampersand2 => Some(Self::LogicalAnd),
            Token::Pipe => Some(Self::BitwiseOr),
            Token::PipeEqual => Some(Self::BitwiseOrAssign),
            Token::Pipe2 => Some(Self::LogicalOr),
            Token::Caret => Some(Self::BitwiseXor),
            Token::CaretEqual => Some(Self::BitwiseXorAssign),
            Token::BangEqual => Some(Self::NotEqual),
            Token::Equal => Some(Self::Assign),
            Token::Equal2 => Some(Self::Equal),
            Token::Dot => Some(Self::Access),
            Token::SquareLeft => Some(Self::Subscript),
            Token::AngleLeft => Some(Self::LessThan),
            Token::AngleLeftEqual => Some(Self::LessEqual),
            Token::AngleLeft2 => Some(Self::ShiftLeft),
            Token::AngleLeft2Equal => Some(Self::ShiftLeftAssign),
            Token::AngleRight => Some(Self::GreaterThan),
            Token::AngleRightEqual => Some(Self::GreaterEqual),
            Token::AngleRight2 => Some(Self::ShiftRight),
            Token::AngleRight2Equal => Some(Self::ShiftRightAssign),
            Token::As => Some(Self::Convert),
            _ => None
        }
    }

    pub fn notation(&self) -> &'static str {
        match self {
            Self::Subscript => "[", // FIXME
            Self::Access => ".",
            Self::DerefAccess => "->",
            Self::Convert => " as ",
            Self::Multiply => " * ",
            Self::Divide => " / ",
            Self::Remainder => " % ",
            Self::Add => " + ",
            Self::Subtract => " - ",
            Self::ShiftLeft => " << ",
            Self::ShiftRight => " >> ",
            Self::LessThan => " < ",
            Self::LessEqual => " <= ",
            Self::GreaterThan => " > ",
            Self::GreaterEqual => " >= ",
            Self::Equal => " == ",
            Self::NotEqual => " != ",
            Self::BitwiseAnd => " & ",
            Self::BitwiseXor => " ^ ",
            Self::BitwiseOr => " | ",
            Self::LogicalAnd => " && ",
            Self::LogicalOr => " || ",
            Self::Assign => " = ",
            Self::MultiplyAssign => " *= ",
            Self::DivideAssign => " /= ",
            Self::RemainderAssign => " %= ",
            Self::AddAssign => " += ",
            Self::SubtractAssign => " -= ",
            Self::ShiftLeftAssign => " <<= ",
            Self::ShiftRightAssign => " >>= ",
            Self::BitwiseAndAssign => " &= ",
            Self::BitwiseXorAssign => " ^= ",
            Self::BitwiseOrAssign => " |= ",
        }
    }
}

impl fmt::Display for BinaryOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.notation())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Operation {
    Unary(UnaryOperation),
    Binary(BinaryOperation),
}

impl Operation {
    pub fn precedence(&self) -> Precedence {
        match self {
            Self::Unary(unary) => unary.precedence(),
            Self::Binary(binary) => binary.precedence(),
        }
    }

    pub fn associativity(&self) -> Associativity {
        match self {
            Self::Unary(unary) => unary.associativity(),
            Self::Binary(binary) => binary.associativity(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValueType {
    Named(String),
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => {
                write!(f, "{name}")
            },
        }
    }
}

#[derive(Debug)]
pub enum Node {
    Literal(Literal),
    Unary {
        operation: UnaryOperation,
        operand: Box<Node>,
    },
    Binary {
        operation: BinaryOperation,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Scope {
        statements: Vec<Box<Node>>,
    },
    Let {
        identifier: Box<Node>,
        value_type: ValueType,
        value: Option<Box<Node>>,
    },
    Conditional {
        condition: Box<Node>,
        consequent: Box<Node>,
        alternative: Option<Box<Node>>,
    },
    While {
        condition: Box<Node>,
        body: Box<Node>,
    },
    Break,
    Continue,
    Return {
        value: Option<Box<Node>>,
    },
    Print {
        value: Box<Node>,
    },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(literal) => {
                write!(f, "{literal}")
            },
            Self::Unary { operation, operand } => {
                write!(f, "({operation}{operand})")
            },
            Self::Binary { operation, lhs, rhs } => {
                write!(f, "({lhs}{operation}{rhs})")
            },
            Self::Scope { statements } => {
                write!(f, " {{")?;
                for statement in statements {
                    statement.fmt(f)?;
                }
                write!(f, " }}")
            },
            Self::Let { identifier, value_type, value } => {
                if let Some(value) = value {
                    write!(f, " let {identifier}: {value_type} = {value};")
                } else {
                    write!(f, " let {identifier}: {value_type};")
                }
            },
            Self::Conditional { condition, consequent, alternative } => {
                if let Some(alternative) = alternative {
                    write!(f, " if ({condition}) {consequent} else {alternative}")
                } else {
                    write!(f, " if ({condition}) {consequent}")
                }
            },
            Self::While { condition, body } => {
                write!(f, " while ({condition}) {body}")
            }
            Self::Break => {
                write!(f, " break;")
            },
            Self::Continue => {
                write!(f, " continue;")
            },
            Self::Return { value } => {
                if let Some(value) = value {
                    write!(f, " return {value};")
                } else {
                    write!(f, " return;")
                }
            }
            Self::Print { value } => {
                write!(f, " print {value};")
            },
        }
    }
}
