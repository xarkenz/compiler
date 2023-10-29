pub mod info;
pub mod llvm;

use crate::{Error, FileError, RawError};
use crate::token;
use crate::ast;

use std::io::{Write, BufRead};
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum ValueFormat {
    Integer(usize),
    Pointer(Box<ValueFormat>),
}

impl ValueFormat {
    pub fn pointer(self) -> Self {
        Self::Pointer(Box::new(self))
    }
}

impl fmt::Display for ValueFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(bits) => write!(f, "i{bits}"),
            Self::Pointer(inner) => write!(f, "{inner}*")
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConstantValue {
    Signed32(i32),
    Unsigned32(u32),
}

impl ConstantValue {
    pub fn format(&self) -> ValueFormat {
        match self {
            Self::Signed32(_) => ValueFormat::Integer(32),
            Self::Unsigned32(_) => ValueFormat::Integer(32),
        }
    }
}

impl fmt::Display for ConstantValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signed32(value) => value.fmt(f),
            Self::Unsigned32(value) => value.fmt(f),
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

    pub fn generate_node_llvm(&mut self, node: &ast::Node) -> crate::Result<Option<RightValue>> {
        match node {
            ast::Node::Literal(literal) => {
                match literal {
                    token::Literal::Identifier(name) => {
                        // If we don't clone here, with the way things are currently set up, we can't borrow self.emitter as mutable
                        let symbol = self.get_symbol(name)?.clone();
                        let output = self.next_anonymous_register(symbol.format().clone());

                        llvm::emit_symbol_load(&mut self.emitter, &output, &symbol)
                            .map_err(|cause| self.file_error(cause))?;

                        Ok(Some(RightValue::Register(output)))
                    },
                    token::Literal::Integer(value) => {
                        Ok(Some(RightValue::Constant(ConstantValue::Signed32(*value as i32))))
                    },
                }
            },
            ast::Node::Unary { operation, operand } => {
                let operand = self.generate_node_llvm(operand.as_ref())?
                    .ok_or_else(|| self.error(format!("operation '{operation}x' expects a value for x")))?;
                let output = self.next_anonymous_register(operand.format());
                let _ = output; // temporary

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
                let output = self.next_anonymous_register(lhs.format());

                match operation {
                    ast::BinaryOperation::Add => {
                        llvm::emit_addition(&mut self.emitter, &output, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Subtract => {
                        llvm::emit_subtraction(&mut self.emitter, &output, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Multiply => {
                        llvm::emit_multiplication(&mut self.emitter, &output, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Divide => {
                        llvm::emit_division(&mut self.emitter, &output, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    _ => return Err(self.error(format!("operation 'x{operation}y' not yet implemented")))
                }

                Ok(Some(RightValue::Register(output)))
            },
            ast::Node::Let { identifier, value_type, value } => {
                if let ast::Node::Literal(token::Literal::Identifier(name)) = identifier.as_ref() {
                    // TODO: parse value_type to get the format instead of just ignoring it lol
                    let _ = value_type; // temporary
                    let format = ValueFormat::Integer(32);
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
                let output_register = self.next_anonymous_register(ValueFormat::Integer(32));

                llvm::emit_print_i32(&mut self.emitter, &output_register, &value_to_print)
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
