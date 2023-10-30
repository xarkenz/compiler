pub mod info;
pub mod llvm;

use crate::{Error, FileError, RawError};
use crate::token;
use crate::ast;

use std::io::{Write, BufRead};
use std::fmt;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IntegerSemantics {
    Signed,
    Unsigned,
    Boolean,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValueFormat {
    Integer {
        bits: usize,
        semantics: IntegerSemantics,
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
                    "bool" => Ok(ValueFormat::Integer { bits: 1, semantics: IntegerSemantics::Boolean }),
                    "i8" => Ok(ValueFormat::Integer { bits: 8, semantics: IntegerSemantics::Signed }),
                    "u8" => Ok(ValueFormat::Integer { bits: 8, semantics: IntegerSemantics::Unsigned }),
                    "i16" => Ok(ValueFormat::Integer { bits: 16, semantics: IntegerSemantics::Signed }),
                    "u16" => Ok(ValueFormat::Integer { bits: 16, semantics: IntegerSemantics::Unsigned }),
                    "i32" => Ok(ValueFormat::Integer { bits: 32, semantics: IntegerSemantics::Signed }),
                    "u32" => Ok(ValueFormat::Integer { bits: 32, semantics: IntegerSemantics::Unsigned }),
                    "i64" => Ok(ValueFormat::Integer { bits: 64, semantics: IntegerSemantics::Signed }),
                    "u64" => Ok(ValueFormat::Integer { bits: 64, semantics: IntegerSemantics::Unsigned }),
                    _ => Err(RawError::new(format!("unrecognized type name 'name'")).into_boxed())
                }
            },
        }
    }
}

impl fmt::Display for ValueFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer { bits, .. } => write!(f, "i{bits}"),
            Self::Pointer { to } => write!(f, "{to}*")
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConstantValue {
    Bool(bool),
    Signed8(i8),
    Unsigned8(u8),
    Signed16(i16),
    Unsigned16(u16),
    Signed32(i32),
    Unsigned32(u32),
    Signed64(i64),
    Unsigned64(u64),
}

impl ConstantValue {
    pub fn format(&self) -> ValueFormat {
        match self {
            Self::Bool(_) => ValueFormat::Integer { bits: 1, semantics: IntegerSemantics::Boolean },
            Self::Signed8(_) => ValueFormat::Integer { bits: 8, semantics: IntegerSemantics::Signed },
            Self::Unsigned8(_) => ValueFormat::Integer { bits: 8, semantics: IntegerSemantics::Unsigned },
            Self::Signed16(_) => ValueFormat::Integer { bits: 16, semantics: IntegerSemantics::Signed },
            Self::Unsigned16(_) => ValueFormat::Integer { bits: 16, semantics: IntegerSemantics::Unsigned },
            Self::Signed32(_) => ValueFormat::Integer { bits: 32, semantics: IntegerSemantics::Signed },
            Self::Unsigned32(_) => ValueFormat::Integer { bits: 32, semantics: IntegerSemantics::Unsigned },
            Self::Signed64(_) => ValueFormat::Integer { bits: 64, semantics: IntegerSemantics::Signed },
            Self::Unsigned64(_) => ValueFormat::Integer { bits: 64, semantics: IntegerSemantics::Unsigned },
        }
    }
}

impl fmt::Display for ConstantValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => value.fmt(f),
            Self::Signed8(value) => value.fmt(f),
            Self::Unsigned8(value) => value.fmt(f),
            Self::Signed16(value) => value.fmt(f),
            Self::Unsigned16(value) => value.fmt(f),
            Self::Signed32(value) => value.fmt(f),
            Self::Unsigned32(value) => value.fmt(f),
            Self::Signed64(value) => value.fmt(f),
            Self::Unsigned64(value) => value.fmt(f),
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

    pub fn change_format(&mut self, value: RightValue, to_format: &ValueFormat) -> crate::Result<RightValue> {
        let from_format = value.format();
        if let (
            ValueFormat::Integer { bits: to_bits, .. },
            ValueFormat::Integer { bits: from_bits, .. },
        ) = (to_format, &from_format) {
            if to_bits > from_bits {
                let result = self.next_anonymous_register(to_format.clone());
                llvm::emit_extension(&mut self.emitter, &result, &value)
                    .map_err(|cause| self.file_error(cause))?;

                Ok(RightValue::Register(result))
            }
            else if to_bits < from_bits {
                let result = self.next_anonymous_register(to_format.clone());
                llvm::emit_truncation(&mut self.emitter, &result, &value)
                    .map_err(|cause| self.file_error(cause))?;

                Ok(RightValue::Register(result))
            }
            else {
                Ok(value)
            }
        }
        else if to_format == &from_format {
            Ok(value)
        }
        else {
            Err(self.error(format!("cannot convert from {from_format} to {to_format}")))
        }
    }

    pub fn combined_format(&self, lhs_format: ValueFormat, rhs_format: ValueFormat) -> crate::Result<ValueFormat> {
        if let (
            ValueFormat::Integer { bits: lhs_bits, semantics: lhs_semantics },
            ValueFormat::Integer { bits: rhs_bits, semantics: rhs_semantics },
        ) = (&lhs_format, &rhs_format) {
            let semantics = if lhs_semantics == &IntegerSemantics::Signed || rhs_semantics == &IntegerSemantics::Signed {
                IntegerSemantics::Signed
            } else {
                IntegerSemantics::Unsigned
            };

            Ok(ValueFormat::Integer {
                bits: *lhs_bits.max(rhs_bits),
                semantics,
            })
        }
        else {
            Err(self.error(format!("cannot convert between {lhs_format} and {rhs_format}")))
        }
    }

    pub fn generate_node_llvm(&mut self, node: &ast::Node) -> crate::Result<Option<RightValue>> {
        match node {
            ast::Node::Literal(literal) => {
                match literal {
                    token::Literal::Identifier(name) => {
                        // If we don't clone here, with the way things are currently set up, we can't borrow self.emitter as mutable
                        let symbol = self.get_symbol(name)?.clone();
                        let result = self.next_anonymous_register(symbol.format().clone());

                        llvm::emit_symbol_load(&mut self.emitter, &result, &symbol)
                            .map_err(|cause| self.file_error(cause))?;

                        Ok(Some(RightValue::Register(result)))
                    },
                    token::Literal::Integer(value) => {
                        Ok(Some(RightValue::Constant(ConstantValue::Signed32(*value as i32))))
                    },
                }
            },
            ast::Node::Unary { operation, operand } => {
                let operand = self.generate_node_llvm(operand.as_ref())?
                    .ok_or_else(|| self.error(format!("operation '{operation}x' expects a value for x")))?;
                let result = self.next_anonymous_register(operand.format());
                let _ = result; // temporary

                match operation {
                    _ => return Err(self.error(format!("operation '{operation}x' not yet implemented")))
                }

                // Ok(Some(RightValue::Register(output)))
            },
            ast::Node::Binary { operation: ast::BinaryOperation::Assign, lhs, rhs } => {
                if let ast::Node::Literal(token::Literal::Identifier(name)) = lhs.as_ref() {
                    let rhs = self.generate_node_llvm(rhs.as_ref())?
                        .ok_or_else(|| self.error(format!("operation 'x = y' expects a value for y")))?;
                    // If we don't clone here, with the way things are currently set up, we can't borrow self.emitter as mutable
                    let symbol = self.get_symbol(name)?.clone();

                    llvm::emit_symbol_store(&mut self.emitter, &rhs, &symbol)
                        .map_err(|cause| self.file_error(cause))?;

                    Ok(Some(rhs))
                }
                else {
                    Err(self.error(String::from("invalid left-hand side for '='")))
                }
            },
            ast::Node::Binary { operation, lhs, rhs } => {
                let lhs = self.generate_node_llvm(lhs.as_ref())?
                    .ok_or_else(|| self.error(format!("operation 'x{operation}y' expects a value for x")))?;
                let rhs = self.generate_node_llvm(rhs.as_ref())?
                    .ok_or_else(|| self.error(format!("operation 'x{operation}y' expects a value for y")))?;
                let result;

                match operation {
                    ast::BinaryOperation::Add => {
                        result = self.next_anonymous_register(self.combined_format(lhs.format(), rhs.format())?);

                        let lhs = self.change_format(lhs, result.format())?;
                        let lhs = self.change_format(lhs, result.format())?;

                        llvm::emit_addition(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Subtract => {
                        result = self.next_anonymous_register(self.combined_format(lhs.format(), rhs.format())?);

                        let lhs = self.change_format(lhs, result.format())?;
                        let lhs = self.change_format(lhs, result.format())?;

                        llvm::emit_subtraction(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Multiply => {
                        result = self.next_anonymous_register(self.combined_format(lhs.format(), rhs.format())?);

                        let lhs = self.change_format(lhs, result.format())?;
                        let lhs = self.change_format(lhs, result.format())?;

                        llvm::emit_multiplication(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Divide => {
                        result = self.next_anonymous_register(self.combined_format(lhs.format(), rhs.format())?);

                        let lhs = self.change_format(lhs, result.format())?;
                        let lhs = self.change_format(lhs, result.format())?;

                        llvm::emit_division(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Equal => {
                        result = self.next_anonymous_register(ValueFormat::Integer { bits: 1, semantics: IntegerSemantics::Boolean });

                        llvm::emit_cmp_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::NotEqual => {
                        result = self.next_anonymous_register(ValueFormat::Integer { bits: 1, semantics: IntegerSemantics::Boolean });

                        llvm::emit_cmp_not_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::LessThan => {
                        result = self.next_anonymous_register(ValueFormat::Integer { bits: 1, semantics: IntegerSemantics::Boolean });

                        llvm::emit_cmp_less_than(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::LessEqual => {
                        result = self.next_anonymous_register(ValueFormat::Integer { bits: 1, semantics: IntegerSemantics::Boolean });

                        llvm::emit_cmp_less_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::GreaterThan => {
                        result = self.next_anonymous_register(ValueFormat::Integer { bits: 1, semantics: IntegerSemantics::Boolean });

                        llvm::emit_cmp_greater_than(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::GreaterEqual => {
                        result = self.next_anonymous_register(ValueFormat::Integer { bits: 1, semantics: IntegerSemantics::Boolean });

                        llvm::emit_cmp_greater_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    _ => return Err(self.error(format!("operation 'x{operation}y' not yet implemented")))
                }

                Ok(Some(RightValue::Register(result)))
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
                    let symbol = info::Symbol::new(name.clone(), format, alignment, register);

                    llvm::emit_symbol_declaration(&mut self.emitter, &symbol)
                        .map_err(|cause| self.file_error(cause))?;

                    if let Some(node) = value {
                        let value = self.generate_node_llvm(node.as_ref())?
                            .ok_or_else(|| self.error(String::from("'let' expects a value")))?;

                        llvm::emit_symbol_store(&mut self.emitter, &value, &symbol)
                            .map_err(|cause| self.file_error(cause))?;
                    }

                    self.local_symbols.insert(symbol);

                    Ok(None)
                }
                else {
                    Err(self.error(String::from("invalid left-hand side for 'let'")))
                }
            },
            ast::Node::Print { value } => {
                let value_to_print = self.generate_node_llvm(value.as_ref())?
                    .ok_or_else(|| self.error(String::from("'print' expects a value")))?;
                let to_format = match value_to_print.format() {
                    ValueFormat::Integer { semantics, .. } => ValueFormat::Integer { bits: 64, semantics },
                    format => format
                };
                let value_to_print = self.change_format(value_to_print, &to_format)?;
                let result_register = self.next_anonymous_register(ValueFormat::Integer { bits: 32, semantics: IntegerSemantics::Signed });

                llvm::emit_print(&mut self.emitter, &result_register, &value_to_print)
                    .map_err(|cause| self.file_error(cause))?;

                Ok(None)
            },
        }
    }

    pub fn generate_llvm(mut self, parser: &mut ast::parse::Parser<'a, impl BufRead>) -> crate::Result<()> {
        llvm::emit_preamble(&mut self.emitter, self.filename)
            .map_err(|cause| self.file_error(cause))?;
        
        while let Some(statement) = parser.parse_statement()? {
            self.generate_node_llvm(statement.as_ref())?;
        }

        llvm::emit_return(&mut self.emitter, &RightValue::Constant(ConstantValue::Signed32(0)))
            .map_err(|cause| self.file_error(cause))?;

        llvm::emit_postamble(&mut self.emitter)
            .map_err(|cause| self.file_error(cause))
    }
}
