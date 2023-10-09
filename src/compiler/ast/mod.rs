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
            .map(|cmp| cmp.reverse())
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
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(literal) => write!(f, "{literal}"),
            Self::Unary { operation, operand } => write!(f, "({operation}{operand:?})"),
            Self::Binary { operation, lhs, rhs } => write!(f, "({lhs:?}{operation}{rhs:?})"),
        }
    }
}
