pub mod info;
pub mod llvm;

use crate::{Error, FileError, RawError};
use crate::token;
use crate::ast;

use std::io::{Write, BufRead};
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum ValueFormat {
    Signed(usize),
    Unsigned(usize),
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
            Self::Signed(bits) => write!(f, "i{bits}"),
            Self::Unsigned(bits) => write!(f, "u{bits}"),
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
            Self::Signed32(_) => ValueFormat::Signed(32),
            Self::Unsigned32(_) => ValueFormat::Unsigned(32),
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
    id: usize,
    format: ValueFormat,
}

impl Register {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn format(&self) -> &ValueFormat {
        &self.format
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.id)
    }
}

#[derive(Clone, Debug)]
pub struct StackEntry {
    format: ValueFormat,
    alignment: usize,
    register: Register,
    is_free: bool,
    loaded_register: Option<Register>,
}

impl StackEntry {
    pub fn format(&self) -> &ValueFormat {
        &self.format
    }

    pub fn alignment(&self) -> usize {
        self.alignment
    }

    pub fn register(&self) -> &Register {
        &self.register
    }

    pub fn is_free(&self) -> bool {
        self.is_free
    }

    pub fn set_free(&mut self, is_free: bool) {
        self.is_free = is_free;
    }

    pub fn loaded_register(&self) -> Option<&Register> {
        self.loaded_register.as_ref()
    }

    pub fn set_loaded_register(&mut self, register: Option<Register>) {
        self.loaded_register = register;
    }
}

#[derive(Debug)]
pub struct Generator<'a, T: Write> {
    filename: &'a str,
    emitter: T,
    next_register_id: usize,
    stack_entries: Vec<StackEntry>,
}

impl<'a> Generator<'a, std::fs::File> {
    pub fn from_filename(filename: &'a str) -> crate::Result<Self> {
        std::fs::File::create(filename).map(|file| Self::new(filename, file))
            .map_err(|cause| FileError::new(filename.to_owned(), None, cause).into_boxed())
    }
}

impl<'a, T: Write> Generator<'a, T> {
    pub fn new(filename: &'a str, emitter: T) -> Self {
        Self {
            filename,
            emitter,
            next_register_id: 1,
            stack_entries: Vec::new(),
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

    fn new_register(&mut self, format: ValueFormat) -> Register {
        let id = self.next_register_id;
        self.next_register_id += 1;
        Register {
            id,
            format,
        }
    }

    fn next_free_stack_entry_index(&mut self, format: &ValueFormat) -> Option<usize> {
        // Return the index of the next free stack entry which has the specified format
        self.stack_entries.iter().position(|entry| entry.is_free() && entry.format() == format)
    }

    fn allocate_stack_entries(&mut self, node: &ast::Node) -> crate::Result<()> {
        match node {
            ast::Node::Unary { operand, .. } => {
                self.allocate_stack_entries(operand.as_ref())?;
            },
            ast::Node::Binary { lhs, rhs, .. } => {
                self.allocate_stack_entries(lhs.as_ref())?;
                self.allocate_stack_entries(rhs.as_ref())?;
            },
            ast::Node::Literal(_) => {
                let register = self.new_register(ValueFormat::Signed(32).pointer());
                self.stack_entries.push(StackEntry {
                    format: ValueFormat::Signed(32),
                    alignment: 4,
                    register,
                    is_free: true,
                    loaded_register: None,
                });
            },
        }
        Ok(())
    }

    fn load_register_if_unloaded(&mut self, register: Register) -> crate::Result<Register> {
        // Try finding the stack entry this register corresponds to
        if let Some(index) = self.stack_entries.iter().position(|entry| entry.register() == &register) {
            if self.stack_entries[index].loaded_register().is_none() {
                // The entry hasn't been loaded; create a new register and load it
                let register_to_load = self.new_register(self.stack_entries[index].format().clone());
                llvm::emit_load_register(&mut self.emitter, &register_to_load, &self.stack_entries[index])
                    .map_err(|cause| self.file_error(cause))?;
                self.stack_entries[index].set_loaded_register(Some(register_to_load));
                // Safe to unwrap because we just set it to Some
            }
            Ok(self.stack_entries[index].loaded_register().unwrap().clone())
        } else {
            // The register doesn't correspond to a stack entry; it is considered "loaded" for our purposes
            Ok(register)
        }
    }

    fn generate_ast_llvm(&mut self, node: &ast::Node) -> crate::Result<Register> {
        match node {
            ast::Node::Unary { operation, operand } => {
                let operand_register = self.generate_ast_llvm(operand.as_ref())?;
                let operand_register = self.load_register_if_unloaded(operand_register)?;
                let output_register = self.new_register(ValueFormat::Signed(32));

                match operation {
                    _ => Err(self.error(format!("operation '{operation}x' not implemented yet")))
                }
            },
            ast::Node::Binary { operation, lhs, rhs } => {
                let lhs_register = self.generate_ast_llvm(lhs.as_ref())?;
                let rhs_register = self.generate_ast_llvm(rhs.as_ref())?;
                let lhs_register = self.load_register_if_unloaded(lhs_register)?;
                let rhs_register = self.load_register_if_unloaded(rhs_register)?;
                let output_register = self.new_register(ValueFormat::Signed(32));

                match operation {
                    ast::BinaryOperation::Add => {
                        llvm::emit_addition(&mut self.emitter, &output_register, &lhs_register, &rhs_register)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Subtract => {
                        llvm::emit_subtraction(&mut self.emitter, &output_register, &lhs_register, &rhs_register)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Multiply => {
                        llvm::emit_multiplication(&mut self.emitter, &output_register, &lhs_register, &rhs_register)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    ast::BinaryOperation::Divide => {
                        llvm::emit_division(&mut self.emitter, &output_register, &lhs_register, &rhs_register)
                            .map_err(|cause| self.file_error(cause))?;
                    },
                    _ => return Err(self.error(format!("operation 'x{operation}y' not implemented yet")))
                }
                Ok(output_register)
            },
            ast::Node::Literal(literal) => {
                match literal {
                    token::Literal::Integer(value) => {
                        let value = ConstantValue::Signed32(*value as i32);
                        let index = self.next_free_stack_entry_index(&value.format())
                            .ok_or_else(|| self.error(format!("no suitable stack entry free for storing {value}")))?;
                        llvm::emit_store_constant(&mut self.emitter, &value, &self.stack_entries[index])
                            .map_err(|cause| self.file_error(cause))?;
                        self.stack_entries[index].set_free(false);
                        Ok(self.stack_entries[index].register().clone())
                    },
                    _ => Err(self.error(String::from("only integer literals allowed for now")))
                }
            },
        }
    }

    pub fn generate_llvm(mut self, parser: &mut ast::parse::Parser<'a, impl BufRead>) -> crate::Result<()> {
        llvm::emit_preamble(&mut self.emitter, self.filename)
            .map_err(|cause| self.file_error(cause))?;
        
        while let Some(statement) = parser.parse_statement()? {
            self.allocate_stack_entries(statement.as_ref())?;
            llvm::emit_stack_allocations(&mut self.emitter, &self.stack_entries)
                .map_err(|cause| self.file_error(cause))?;

            let print_register = self.generate_ast_llvm(statement.as_ref())?;
            let print_register = self.load_register_if_unloaded(print_register)?;
            let print_target_register = self.new_register(ValueFormat::Signed(32));
            llvm::emit_print_i32(&mut self.emitter, &print_target_register, &print_register)
                .map_err(|cause| self.file_error(cause))?;

            self.stack_entries.clear(); // TODO: refactor...
        }

        llvm::emit_postamble(&mut self.emitter)
            .map_err(|cause| self.file_error(cause))
    }
}
