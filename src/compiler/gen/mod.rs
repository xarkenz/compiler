pub mod info;
pub mod llvm;

use crate::{Error, FileError, RawError};
use crate::token;
use crate::ast;

use std::io::{Write, BufRead};
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum ValueFormat {
    Boolean,
    Integer {
        size: usize,
        signed: bool,
    },
    Pointer {
        to: Box<ValueFormat>,
    },
}

impl ValueFormat {
    pub fn pointer(self) -> Self {
        Self::Pointer {
            to: Box::new(self),
        }
    }
}

impl TryFrom<&ast::ValueType> for ValueFormat {
    type Error = Box<dyn Error>;

    fn try_from(value: &ast::ValueType) -> crate::Result<Self> {
        match value {
            ast::ValueType::Named(name) => {
                match name.as_str() {
                    "bool" => Ok(ValueFormat::Boolean),
                    "i8" => Ok(ValueFormat::Integer { size: 1, signed: true }),
                    "u8" => Ok(ValueFormat::Integer { size: 1, signed: false }),
                    "i16" => Ok(ValueFormat::Integer { size: 2, signed: true }),
                    "u16" => Ok(ValueFormat::Integer { size: 2, signed: false }),
                    "i32" => Ok(ValueFormat::Integer { size: 4, signed: true }),
                    "u32" => Ok(ValueFormat::Integer { size: 4, signed: false }),
                    "i64" => Ok(ValueFormat::Integer { size: 8, signed: true }),
                    "u64" => Ok(ValueFormat::Integer { size: 8, signed: false }),
                    _ => Err(RawError::new(format!("unrecognized type name 'name'")).into_boxed())
                }
            },
        }
    }
}

impl fmt::Display for ValueFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => write!(f, "i1"),
            Self::Integer { size, .. } => write!(f, "i{bits}", bits = size * 8),
            Self::Pointer { to } => write!(f, "{to}*"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConstantValue {
    Boolean(bool),
    Integer(u64, ValueFormat),
}

impl ConstantValue {
    pub fn format(&self) -> ValueFormat {
        match self {
            Self::Boolean(_) => ValueFormat::Boolean,
            Self::Integer(_, format) => format.clone(),
        }
    }
}

impl fmt::Display for ConstantValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(value) => value.fmt(f),
            Self::Integer(value, _) => value.fmt(f),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Register {
    name: String,
    format: ValueFormat,
    is_global: bool,
}

impl Register {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn format(&self) -> &ValueFormat {
        &self.format
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_global() {
            write!(f, "@{}", self.name)
        } else {
            write!(f, "%{}", self.name)
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum RightValue {
    Constant(ConstantValue),
    Register(Register),
}

impl RightValue {
    pub fn format(&self) -> ValueFormat {
        match self {
            Self::Constant(value) => value.format(),
            Self::Register(value) => value.format().clone(),
        }
    }
}

impl fmt::Display for RightValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(value) => value.fmt(f),
            Self::Register(value) => value.fmt(f),
        }
    }
}

pub fn expected_unary_operand_format(operation: ast::UnaryOperation, expected_format: Option<ValueFormat>) -> Option<ValueFormat> {
    match operation {
        ast::UnaryOperation::Positive => expected_format,
        ast::UnaryOperation::Negative => expected_format,
        ast::UnaryOperation::BitwiseNot => expected_format,
        ast::UnaryOperation::LogicalNot => Some(ValueFormat::Boolean),
        _ => None
    }
}

pub fn expected_binary_lhs_format(operation: ast::BinaryOperation, expected_format: Option<ValueFormat>) -> Option<ValueFormat> {
    match operation {
        ast::BinaryOperation::Multiply => expected_format,
        ast::BinaryOperation::Divide => expected_format,
        ast::BinaryOperation::Remainder => expected_format,
        ast::BinaryOperation::Add => expected_format,
        ast::BinaryOperation::Subtract => expected_format,
        ast::BinaryOperation::ShiftLeft => expected_format,
        ast::BinaryOperation::ShiftRight => expected_format,
        ast::BinaryOperation::BitwiseAnd => expected_format,
        ast::BinaryOperation::BitwiseXor => expected_format,
        ast::BinaryOperation::BitwiseOr => expected_format,
        ast::BinaryOperation::LogicalAnd => Some(ValueFormat::Boolean),
        ast::BinaryOperation::LogicalOr => Some(ValueFormat::Boolean),
        _ => None
    }
}

pub fn expected_binary_rhs_format(operation: ast::BinaryOperation, expected_format: Option<ValueFormat>, lhs_format: Option<ValueFormat>) -> Option<ValueFormat> {
    match operation {
        ast::BinaryOperation::Multiply => lhs_format,
        ast::BinaryOperation::Divide => lhs_format,
        ast::BinaryOperation::Remainder => lhs_format,
        ast::BinaryOperation::Add => lhs_format,
        ast::BinaryOperation::Subtract => lhs_format,
        ast::BinaryOperation::LessThan => lhs_format,
        ast::BinaryOperation::LessEqual => lhs_format,
        ast::BinaryOperation::GreaterThan => lhs_format,
        ast::BinaryOperation::GreaterEqual => lhs_format,
        ast::BinaryOperation::Equal => lhs_format,
        ast::BinaryOperation::NotEqual => lhs_format,
        ast::BinaryOperation::BitwiseAnd => lhs_format,
        ast::BinaryOperation::BitwiseXor => lhs_format,
        ast::BinaryOperation::BitwiseOr => lhs_format,
        ast::BinaryOperation::LogicalAnd => Some(ValueFormat::Boolean),
        ast::BinaryOperation::LogicalOr => Some(ValueFormat::Boolean),
        _ => None
    }
}

#[derive(Debug)]
pub struct Generator<'a, T: Write> {
    filename: &'a str,
    emitter: T,
    next_anonymous_register_id: usize,
    global_symbols: info::SymbolTable,
    local_symbols: info::SymbolTable,
}

impl<'a> Generator<'a, std::fs::File> {
    pub fn from_filename(filename: &'a str) -> crate::Result<Self> {
        std::fs::File::create(filename).map(|file| Self::new(filename, file))
            .map_err(|cause| FileError::new(filename.to_owned(), None, cause).into_boxed())
    }
}

impl<'a, T: Write> Generator<'a, T> {
    const DEFAULT_SYMBOL_TABLE_CAPACITY: usize = 256;

    pub fn new(filename: &'a str, emitter: T) -> Self {
        Self {
            filename,
            emitter,
            next_anonymous_register_id: 1,
            global_symbols: info::SymbolTable::new(Self::DEFAULT_SYMBOL_TABLE_CAPACITY),
            local_symbols: info::SymbolTable::new(Self::DEFAULT_SYMBOL_TABLE_CAPACITY),
        }
    }

    pub fn filename(&self) -> &'a str {
        self.filename
    }

    pub fn file_error(&self, cause: std::io::Error) -> Box<dyn Error> {
        FileError::new(self.filename.to_owned(), None, cause).into_boxed()
    }

    pub fn error(&self, message: String) -> Box<dyn Error> {
        RawError::new(message).into_boxed()
    }

    pub fn next_anonymous_register(&mut self, format: ValueFormat) -> Register {
        let id = self.next_anonymous_register_id;
        self.next_anonymous_register_id += 1;
        Register {
            name: id.to_string(),
            format,
            is_global: false,
        }
    }

    pub fn get_symbol(&self, name: &str) -> crate::Result<&info::Symbol> {
        self.local_symbols.find(name)
            .or_else(|| self.global_symbols.find(name))
            .ok_or_else(|| self.error(format!("undefined symbol '{name}'")))
    }

    pub fn enforce_format(&self, value: &RightValue, format: &ValueFormat) -> crate::Result<()> {
        let got_format = value.format();

        if &got_format == format {
            Ok(())
        }
        else {
            Err(self.error(format!("expected a value of type {format}, got {got_format} instead")))
        }
    }

    pub fn change_format(&mut self, value: RightValue, to_format: &ValueFormat) -> crate::Result<RightValue> {
        let from_format = value.format();

        if to_format == &from_format {
            Ok(value)
        }
        else if let (
            ValueFormat::Integer { size: from_size, .. },
            ValueFormat::Integer { size: to_size, .. },
        ) = (&from_format, to_format) {
            if to_size > from_size {
                let result = self.next_anonymous_register(to_format.clone());
                llvm::emit_extension(&mut self.emitter, &result, &value)
                    .map_err(|cause| self.file_error(cause))?;

                Ok(RightValue::Register(result))
            }
            else if to_size < from_size {
                let result = self.next_anonymous_register(to_format.clone());
                llvm::emit_truncation(&mut self.emitter, &result, &value)
                    .map_err(|cause| self.file_error(cause))?;

                Ok(RightValue::Register(result))
            }
            else {
                Ok(value)
            }
        }
        else if let (
            ValueFormat::Integer { .. },
            ValueFormat::Boolean,
        ) = (&from_format, to_format) {
            let result = self.next_anonymous_register(ValueFormat::Boolean);
            llvm::emit_cmp_not_equal(&mut self.emitter, &result, &value, &RightValue::Constant(ConstantValue::Integer(0, from_format.clone())))
                .map_err(|cause| self.file_error(cause))?;
            
            Ok(RightValue::Register(result))
        }
        else if let (
            ValueFormat::Boolean,
            ValueFormat::Integer { .. },
        ) = (&from_format, to_format) {
            let result = self.next_anonymous_register(to_format.clone());
            llvm::emit_zero_extension(&mut self.emitter, &result, &value)
                .map_err(|cause| self.file_error(cause))?;
            
            Ok(RightValue::Register(result))
        }
        else {
            Err(self.error(format!("cannot convert from {from_format} to {to_format}")))
        }
    }

    pub fn generate_node(&mut self, node: &ast::Node, expected_format: Option<ValueFormat>) -> crate::Result<Option<RightValue>> {
        let result = match node {
            ast::Node::Literal(literal) => {
                match literal {
                    token::Literal::Identifier(name) => {
                        // If we don't clone here, with the way things are currently set up, we can't borrow self.emitter as mutable
                        let symbol = self.get_symbol(name)?.clone();
                        let result = self.next_anonymous_register(symbol.format().clone());

                        llvm::emit_symbol_load(&mut self.emitter, &result, &symbol)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    token::Literal::Integer(value) => {
                        Some(RightValue::Constant(ConstantValue::Integer(*value, expected_format.clone().unwrap_or(ValueFormat::Integer { size: 4, signed: true }))))
                    },
                    token::Literal::Boolean(value) => {
                        Some(RightValue::Constant(ConstantValue::Boolean(*value)))
                    }
                }
            },
            ast::Node::Unary { operation, operand } => {
                let operand = self.generate_node(operand.as_ref(), expected_unary_operand_format(*operation, expected_format))?
                    .ok_or_else(|| self.error(format!("operation '{operation}x' expects a value for x")))?;
                
                match operation {
                    _ => return Err(self.error(format!("operation '{operation}x' not yet implemented")))
                }
            },
            ast::Node::Binary { operation: ast::BinaryOperation::Assign, lhs, rhs } => {
                if let ast::Node::Literal(token::Literal::Identifier(name)) = lhs.as_ref() {
                    // If we don't clone here, with the way things are currently set up, we can't borrow self.emitter as mutable
                    let symbol = self.get_symbol(name)?.clone();
                    let value = self.generate_node(rhs.as_ref(), Some(symbol.format().clone()))?
                        .ok_or_else(|| self.error(String::from("operation 'x = y' expects a value for y")))?;

                    llvm::emit_symbol_store(&mut self.emitter, &value, &symbol)
                        .map_err(|cause| self.file_error(cause))?;

                    Some(value)
                }
                else {
                    return Err(self.error(String::from("invalid left-hand side for '='")));
                }
            },
            ast::Node::Binary { operation: ast::BinaryOperation::Convert, lhs, rhs } => {
                if let ast::Node::Literal(token::Literal::Identifier(name)) = rhs.as_ref() {
                    // TODO: this isn't great...
                    let to_format = ValueFormat::try_from(&ast::ValueType::Named(name.clone()))?;
                    let value = self.generate_node(lhs.as_ref(), None)?
                        .ok_or_else(|| self.error(String::from("operation 'x as y' expects a value for x")))?;

                    Some(self.change_format(value, &to_format)?)
                }
                else {
                    return Err(self.error(String::from("invalid right-hand side for 'as'")));
                }
            }
            ast::Node::Binary { operation, lhs, rhs } => {
                let lhs = self.generate_node(lhs.as_ref(), expected_binary_lhs_format(*operation, expected_format.clone()))?
                    .ok_or_else(|| self.error(format!("operation 'x{operation}y' expects a value for x")))?;
                let rhs = self.generate_node(rhs.as_ref(), expected_binary_rhs_format(*operation, expected_format.clone(), Some(lhs.format())))?
                    .ok_or_else(|| self.error(format!("operation 'x{operation}y' expects a value for y")))?;

                match operation {
                    ast::BinaryOperation::Add => {
                        let result = self.next_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        llvm::emit_addition(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::Subtract => {
                        let result = self.next_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        llvm::emit_subtraction(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::Multiply => {
                        let result = self.next_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        llvm::emit_multiplication(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::Divide => {
                        let result = self.next_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        llvm::emit_division(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::Equal => {
                        let result = self.next_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::NotEqual => {
                        let result = self.next_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_not_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::LessThan => {
                        let result = self.next_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_less_than(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::LessEqual => {
                        let result = self.next_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_less_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::GreaterThan => {
                        let result = self.next_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_greater_than(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::GreaterEqual => {
                        let result = self.next_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_greater_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    _ => return Err(self.error(format!("operation 'x{operation}y' not yet implemented")))
                }
            },
            ast::Node::Let { identifier, value_type, value } => {
                if let ast::Node::Literal(token::Literal::Identifier(name)) = identifier.as_ref() {
                    let format = ValueFormat::try_from(value_type)?;
                    let alignment = 4;
                    let register = Register {
                        name: name.clone(),
                        format: format.clone().pointer(),
                        is_global: false,
                    };
                    let symbol = info::Symbol::new(name.clone(), format.clone(), alignment, register);

                    llvm::emit_symbol_declaration(&mut self.emitter, &symbol)
                        .map_err(|cause| self.file_error(cause))?;

                    if let Some(node) = value {
                        let value = self.generate_node(node.as_ref(), Some(format))?
                            .ok_or_else(|| self.error(String::from("'let' expects a value")))?;

                        llvm::emit_symbol_store(&mut self.emitter, &value, &symbol)
                            .map_err(|cause| self.file_error(cause))?;
                    }

                    self.local_symbols.insert(symbol);

                    None
                }
                else {
                    return Err(self.error(String::from("invalid left-hand side for 'let'")));
                }
            },
            ast::Node::Return { value } => {
                if let Some(value) = value {
                    let value = self.generate_node(value.as_ref(), None)?
                        .ok_or_else(|| self.error(String::from("'let' expects a value")))?;

                    llvm::emit_return(&mut self.emitter, Some(&value))
                        .map_err(|cause| self.file_error(cause))?;
                }
                else {
                    llvm::emit_return(&mut self.emitter, None)
                        .map_err(|cause| self.file_error(cause))?;
                }

                None
            },
            ast::Node::Print { value } => {
                let value_to_print = self.generate_node(value.as_ref(), None)?
                    .ok_or_else(|| self.error(String::from("'print' expects a value")))?;
                let to_format = match value_to_print.format() {
                    ValueFormat::Boolean => ValueFormat::Integer { size: 8, signed: false },
                    ValueFormat::Integer { signed, .. } => ValueFormat::Integer { size: 8, signed },
                    format => format
                };
                let value_to_print = self.change_format(value_to_print, &to_format)?;
                let result_register = self.next_anonymous_register(ValueFormat::Integer { size: 4, signed: true });

                llvm::emit_print(&mut self.emitter, &result_register, &value_to_print)
                    .map_err(|cause| self.file_error(cause))?;

                None
            },
            _ => return Err(self.error(String::from("node type not yet implemented")))
        };

        if let (Some(expected_format), Some(result_value)) = (&expected_format, &result) {
            self.enforce_format(result_value, expected_format)?;
        }

        Ok(result)
    }

    pub fn generate(mut self, parser: &mut ast::parse::Parser<'a, impl BufRead>) -> crate::Result<()> {
        llvm::emit_preamble(&mut self.emitter, self.filename)
            .map_err(|cause| self.file_error(cause))?;
        
        while let Some(statement) = parser.parse_statement()? {
            self.generate_node(statement.as_ref(), None)?;
        }

        llvm::emit_postamble(&mut self.emitter)
            .map_err(|cause| self.file_error(cause))
    }
}
