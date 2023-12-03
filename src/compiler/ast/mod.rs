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

#[derive(Clone, Debug)]
pub enum TypeNode {
    Named(String),
    Const(Box<TypeNode>),
    Pointer(Box<TypeNode>),
    Array(Box<TypeNode>, Option<Box<Node>>),
}

impl std::fmt::Display for TypeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => {
                write!(f, "{name}")
            },
            Self::Const(const_type) => {
                write!(f, "const {const_type}")
            },
            Self::Pointer(pointee_type) => {
                write!(f, "*{pointee_type}")
            },
            Self::Array(item_type, Some(length)) => {
                write!(f, "[{item_type}; {length}]")
            },
            Self::Array(item_type, None) => {
                write!(f, "[{item_type}]")
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum Node {
    Literal(Literal),
    ValueType(TypeNode),
    Unary {
        operation: UnaryOperation,
        operand: Box<Node>,
    },
    Binary {
        operation: BinaryOperation,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Call {
        callee: Box<Node>,
        arguments: Vec<Box<Node>>,
    },
    Scope {
        statements: Vec<Box<Node>>,
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
    Let {
        name: String,
        value_type: TypeNode,
        value: Option<Box<Node>>,
    },
    Function {
        name: String,
        parameters: Vec<(String, TypeNode)>,
        is_varargs: bool,
        return_type: TypeNode,
        body: Option<Box<Node>>,
    },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(literal) => {
                write!(f, "{literal}")
            },
            Self::ValueType(value_type) => {
                write!(f, "{value_type}")
            },
            Self::Unary { operation, operand } => {
                write!(f, "({operation}{operand})")
            },
            Self::Binary { operation, lhs, rhs } => {
                write!(f, "({lhs}{operation}{rhs})")
            },
            Self::Call { callee, arguments } => {
                write!(f, "({callee}(")?;
                for argument in arguments {
                    write!(f, "{argument}, ")?;
                }
                write!(f, "))")
            },
            Self::Scope { statements } => {
                write!(f, " {{")?;
                for statement in statements {
                    write!(f, "{statement}")?;
                }
                write!(f, " }}")
            },
            Self::Conditional { condition, consequent, alternative } => {
                if let Some(alternative) = alternative {
                    write!(f, " if ({condition}){consequent} else{alternative}")
                } else {
                    write!(f, " if ({condition}){consequent}")
                }
            },
            Self::While { condition, body } => {
                write!(f, " while ({condition}){body}")
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
            },
            Self::Let { name, value_type, value } => {
                if let Some(value) = value {
                    write!(f, " let {name}: {value_type} = {value};")
                } else {
                    write!(f, " let {name}: {value_type};")
                }
            },
            Self::Function { name, parameters, is_varargs, return_type, body } => {
                write!(f, " function {name}(")?;
                let mut parameters_iter = parameters.iter();
                if let Some((parameter_name, parameter_type)) = parameters_iter.next() {
                    write!(f, "{parameter_name}: {parameter_type}")?;
                    for (parameter_name, parameter_type) in parameters_iter {
                        write!(f, ", {parameter_name}: {parameter_type}")?;
                    }
                    if *is_varargs {
                        write!(f, ", ..")?;
                    }
                }
                else if *is_varargs {
                    write!(f, "..")?;
                }
                if let Some(body) = body {
                    write!(f, ") -> {return_type}{body}")
                }
                else {
                    write!(f, ") -> {return_type};")
                }
            },
        }
    }
}
