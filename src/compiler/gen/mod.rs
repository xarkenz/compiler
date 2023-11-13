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
    Function {
        returned: Box<ValueFormat>,
        parameters: Vec<ValueFormat>,
        is_varargs: bool,
    },
}

impl ValueFormat {
    pub fn size(&self) -> usize {
        match self {
            Self::Boolean => 1,
            Self::Integer { size, .. } => *size,
            Self::Pointer { .. } => 8,
            Self::Function { .. } => 8,
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, ValueFormat::Function { .. })
    }

    pub fn into_pointer(self) -> Self {
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
            Self::Function { returned, parameters, is_varargs } => {
                write!(f, "{returned}(")?;
                let mut parameters_iter = parameters.iter();
                if let Some(first) = parameters_iter.next() {
                    write!(f, "{first}")?;
                    for parameter in parameters_iter {
                        write!(f, ", {parameter}")?;
                    }
                    if *is_varargs {
                        write!(f, ", ...")?;
                    }
                }
                else if *is_varargs {
                    write!(f, "...")?;
                }
                write!(f, ")")
            },
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

#[derive(Clone, PartialEq, Debug)]
pub struct Label {
    name: String,
}

impl Label {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.name)
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

pub fn expected_binary_rhs_format(operation: ast::BinaryOperation, _expected_format: Option<ValueFormat>, lhs_format: Option<ValueFormat>) -> Option<ValueFormat> {
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

#[derive(Clone, Debug)]
pub struct ScopeContext {
    is_global: bool,
    break_label: Option<Label>,
    continue_label: Option<Label>,
}

impl ScopeContext {
    pub fn new() -> Self {
        Self {
            is_global: true,
            break_label: None,
            continue_label: None,
        }
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }

    pub fn break_label(&self) -> Option<&Label> {
        self.break_label.as_ref()
    }

    pub fn continue_label(&self) -> Option<&Label> {
        self.continue_label.as_ref()
    }

    pub fn enter_function(mut self) -> Self {
        self.is_global = false;

        self
    }

    pub fn enter_loop(mut self, break_label: Label, continue_label: Label) -> Self {
        self.break_label = Some(break_label);
        self.continue_label = Some(continue_label);

        self
    }
}

#[derive(Debug)]
pub struct Generator<'a, T: Write> {
    filename: &'a str,
    emitter: T,
    next_anonymous_register_id: usize,
    next_anonymous_label_id: usize,
    global_symbols: info::SymbolTable,
    local_symbols: info::SymbolTable,
}

impl<'a> Generator<'a, std::fs::File> {
    pub fn from_filename(filename: &'a str) -> crate::Result<Self> {
        std::fs::File::create(filename)
            .map(|file| Self::new(filename, file))
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
            next_anonymous_label_id: 1,
            global_symbols: info::SymbolTable::new(Self::DEFAULT_SYMBOL_TABLE_CAPACITY, true),
            local_symbols: info::SymbolTable::new(Self::DEFAULT_SYMBOL_TABLE_CAPACITY, false),
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

    pub fn new_anonymous_register(&mut self, format: ValueFormat) -> Register {
        let id = self.next_anonymous_register_id;
        self.next_anonymous_register_id += 1;
        Register {
            name: format!("{id}"),
            format,
            is_global: false,
        }
    }

    pub fn new_anonymous_label(&mut self) -> Label {
        let id = self.next_anonymous_label_id;
        self.next_anonymous_label_id += 1;
        Label {
            name: format!("-L{id}"),
        }
    }

    pub fn create_symbol(&self, context: &ScopeContext, identifier: String, format: ValueFormat, alignment: usize) -> info::Symbol {
        if context.is_global() {
            self.global_symbols.create_symbol(identifier, format, alignment)
        }
        else {
            self.local_symbols.create_symbol(identifier, format, alignment)
        }
    }

    pub fn define_symbol(&mut self, context: &ScopeContext, symbol: info::Symbol) {
        if context.is_global() {
            self.global_symbols.insert(symbol);
        }
        else {
            self.local_symbols.insert(symbol);
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
                let result = self.new_anonymous_register(to_format.clone());
                llvm::emit_extension(&mut self.emitter, &result, &value)
                    .map_err(|cause| self.file_error(cause))?;

                Ok(RightValue::Register(result))
            }
            else if to_size < from_size {
                let result = self.new_anonymous_register(to_format.clone());
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
            let result = self.new_anonymous_register(ValueFormat::Boolean);
            llvm::emit_cmp_not_equal(&mut self.emitter, &result, &value, &RightValue::Constant(ConstantValue::Integer(0, from_format.clone())))
                .map_err(|cause| self.file_error(cause))?;
            
            Ok(RightValue::Register(result))
        }
        else if let (
            ValueFormat::Boolean,
            ValueFormat::Integer { .. },
        ) = (&from_format, to_format) {
            let result = self.new_anonymous_register(to_format.clone());
            llvm::emit_zero_extension(&mut self.emitter, &result, &value)
                .map_err(|cause| self.file_error(cause))?;
            
            Ok(RightValue::Register(result))
        }
        else {
            Err(self.error(format!("cannot convert from {from_format} to {to_format}")))
        }
    }

    pub fn generate_node(&mut self, node: &ast::Node, context: &ScopeContext, expected_format: Option<ValueFormat>) -> crate::Result<Option<RightValue>> {
        let result = match node {
            ast::Node::Literal(literal) => {
                match literal {
                    token::Literal::Identifier(name) => {
                        // If we don't clone here, with the way things are currently set up, we can't borrow self.emitter as mutable
                        let symbol = self.get_symbol(name)?.clone();
                        let result = self.new_anonymous_register(symbol.format().clone());

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
                let _operand = self.generate_node(operand.as_ref(), context, expected_unary_operand_format(*operation, expected_format))?
                    .ok_or_else(|| self.error(format!("operation '{operation}x' expects a value for x")))?;
                
                match operation {
                    _ => return Err(self.error(format!("operation '{operation}x' not yet implemented")))
                }
            },
            ast::Node::Binary { operation: ast::BinaryOperation::Assign, lhs, rhs } => {
                if let ast::Node::Literal(token::Literal::Identifier(name)) = lhs.as_ref() {
                    // If we don't clone here, with the way things are currently set up, we can't borrow self.emitter as mutable
                    let symbol = self.get_symbol(name)?.clone();
                    let value = self.generate_node(rhs.as_ref(), context, Some(symbol.format().clone()))?
                        .ok_or_else(|| self.error(String::from("operation '... = x' expects a value for x")))?;

                    llvm::emit_symbol_store(&mut self.emitter, &value, &symbol)
                        .map_err(|cause| self.file_error(cause))?;

                    Some(value)
                }
                else {
                    return Err(self.error(String::from("invalid left-hand side for '='")));
                }
            },
            ast::Node::Binary { operation: ast::BinaryOperation::Convert, lhs, rhs } => {
                if let ast::Node::ValueType(value_type) = rhs.as_ref() {
                    let to_format = ValueFormat::try_from(value_type)?;
                    let value = self.generate_node(lhs.as_ref(), context, None)?
                        .ok_or_else(|| self.error(String::from("operation 'x as ...' expects a value for x")))?;

                    Some(self.change_format(value, &to_format)?)
                }
                else {
                    return Err(self.error(String::from("invalid right-hand side for 'as'")));
                }
            }
            ast::Node::Binary { operation, lhs, rhs } => {
                let lhs = self.generate_node(lhs.as_ref(), context, expected_binary_lhs_format(*operation, expected_format.clone()))?
                    .ok_or_else(|| self.error(format!("operation 'x{operation}...' expects a value for x")))?;
                let rhs = self.generate_node(rhs.as_ref(), context, expected_binary_rhs_format(*operation, expected_format.clone(), Some(lhs.format())))?
                    .ok_or_else(|| self.error(format!("operation '...{operation}x' expects a value for x")))?;

                match operation {
                    ast::BinaryOperation::Add => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        llvm::emit_addition(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::Subtract => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        llvm::emit_subtraction(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::Multiply => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        llvm::emit_multiplication(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::Divide => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        llvm::emit_division(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::Equal => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::NotEqual => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_not_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::LessThan => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_less_than(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::LessEqual => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_less_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::GreaterThan => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_greater_than(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    ast::BinaryOperation::GreaterEqual => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        llvm::emit_cmp_greater_equal(&mut self.emitter, &result, &lhs, &rhs)
                            .map_err(|cause| self.file_error(cause))?;

                        Some(RightValue::Register(result))
                    },
                    _ => return Err(self.error(format!("operation 'x{operation}y' not yet implemented")))
                }
            },
            ast::Node::Scope { statements } => {
                self.local_symbols.enter_scope();
                for statement in statements {
                    self.generate_node(statement.as_ref(), context, None)?;
                }
                self.local_symbols.leave_scope();

                None
            },
            ast::Node::Conditional { condition, consequent, alternative } => {
                let condition = self.generate_node(condition.as_ref(), context, Some(ValueFormat::Boolean))?
                    .ok_or_else(|| self.error(String::from("'if (<condition>)' expects a boolean value for <condition>")))?;
                let consequent_label = self.new_anonymous_label();
                let alternative_label = self.new_anonymous_label();

                llvm::emit_conditional_branch(&mut self.emitter, &condition, &consequent_label, &alternative_label)
                    .map_err(|cause| self.file_error(cause))?;
                
                if let Some(alternative) = alternative {
                    let tail_label = self.new_anonymous_label();

                    llvm::emit_label(&mut self.emitter, &consequent_label)
                        .map_err(|cause| self.file_error(cause))?;
                    self.generate_node(consequent.as_ref(), context, None)?;
                    llvm::emit_unconditional_branch(&mut self.emitter, &tail_label)
                        .map_err(|cause| self.file_error(cause))?;

                    llvm::emit_label(&mut self.emitter, &alternative_label)
                        .map_err(|cause| self.file_error(cause))?;
                    self.generate_node(alternative.as_ref(), context, None)?;
                    llvm::emit_unconditional_branch(&mut self.emitter, &tail_label)
                        .map_err(|cause| self.file_error(cause))?;

                    llvm::emit_label(&mut self.emitter, &tail_label)
                        .map_err(|cause| self.file_error(cause))?;
                }
                else {
                    llvm::emit_label(&mut self.emitter, &consequent_label)
                        .map_err(|cause| self.file_error(cause))?;
                    self.generate_node(consequent.as_ref(), context, None)?;
                    llvm::emit_unconditional_branch(&mut self.emitter, &alternative_label)
                        .map_err(|cause| self.file_error(cause))?;

                    llvm::emit_label(&mut self.emitter, &alternative_label)
                        .map_err(|cause| self.file_error(cause))?;
                }

                None
            },
            ast::Node::While { condition, body } => {
                let condition_label = self.new_anonymous_label();

                llvm::emit_unconditional_branch(&mut self.emitter, &condition_label)
                    .map_err(|cause| self.file_error(cause))?;

                llvm::emit_label(&mut self.emitter, &condition_label)
                    .map_err(|cause| self.file_error(cause))?;

                let condition = self.generate_node(condition.as_ref(), context, Some(ValueFormat::Boolean))?
                    .ok_or_else(|| self.error(String::from("'while (<condition>)' expects a boolean value for <condition>")))?;
                let body_label = self.new_anonymous_label();
                let tail_label = self.new_anonymous_label();
                let loop_context = context.clone().enter_loop(tail_label.clone(), condition_label.clone());

                llvm::emit_conditional_branch(&mut self.emitter, &condition, &body_label, &tail_label)
                    .map_err(|cause| self.file_error(cause))?;

                llvm::emit_label(&mut self.emitter, &body_label)
                    .map_err(|cause| self.file_error(cause))?;
                self.generate_node(body.as_ref(), &loop_context, None)?;
                llvm::emit_unconditional_branch(&mut self.emitter, &condition_label)
                    .map_err(|cause| self.file_error(cause))?;

                llvm::emit_label(&mut self.emitter, &tail_label)
                    .map_err(|cause| self.file_error(cause))?;

                None
            },
            ast::Node::Break => {
                let break_label = context.break_label()
                    .ok_or_else(|| self.error(String::from("unexpected 'break' outside loop")))?;

                llvm::emit_unconditional_branch(&mut self.emitter, break_label)
                    .map_err(|cause| self.file_error(cause))?;

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                self.next_anonymous_register_id += 1;

                None
            },
            ast::Node::Continue => {
                let continue_label = context.continue_label()
                    .ok_or_else(|| self.error(String::from("unexpected 'continue' outside loop")))?;

                llvm::emit_unconditional_branch(&mut self.emitter, continue_label)
                    .map_err(|cause| self.file_error(cause))?;

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                self.next_anonymous_register_id += 1;

                None
            },
            ast::Node::Return { value } => {
                if let Some(value) = value {
                    let value = self.generate_node(value.as_ref(), context, None)?
                        .ok_or_else(|| self.error(String::from("'return' expects a value")))?;

                    llvm::emit_return(&mut self.emitter, Some(&value))
                        .map_err(|cause| self.file_error(cause))?;
                }
                else {
                    llvm::emit_return(&mut self.emitter, None)
                        .map_err(|cause| self.file_error(cause))?;
                }

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                self.next_anonymous_register_id += 1;

                None
            },
            ast::Node::Print { value } => {
                let value_to_print = self.generate_node(value.as_ref(), context, None)?
                    .ok_or_else(|| self.error(String::from("'print' expects a value")))?;
                let to_format = match value_to_print.format() {
                    ValueFormat::Boolean => ValueFormat::Integer { size: 8, signed: false },
                    ValueFormat::Integer { signed, .. } => ValueFormat::Integer { size: 8, signed },
                    format => format
                };
                let value_to_print = self.change_format(value_to_print, &to_format)?;
                let result_register = self.new_anonymous_register(ValueFormat::Integer { size: 4, signed: true });

                llvm::emit_print(&mut self.emitter, &result_register, &value_to_print)
                    .map_err(|cause| self.file_error(cause))?;

                None
            },
            ast::Node::Let { name, value_type, value } => {
                let format = ValueFormat::try_from(value_type)?;
                let alignment = format.size();
                let symbol = self.create_symbol(context, name.clone(), format.clone(), alignment);

                llvm::emit_symbol_allocation(&mut self.emitter, &symbol)
                    .map_err(|cause| self.file_error(cause))?;

                if let Some(node) = value {
                    let value = self.generate_node(node.as_ref(), context, Some(format))?
                        .ok_or_else(|| self.error(String::from("'let' expects a value")))?;

                    llvm::emit_symbol_store(&mut self.emitter, &value, &symbol)
                        .map_err(|cause| self.file_error(cause))?;
                }

                self.define_symbol(context, symbol);

                None
            },
            ast::Node::Function { name, parameters, return_type, body } => {
                let returned = Box::new(ValueFormat::try_from(return_type)?);
                let alignment = returned.size(); // TODO: what would this even mean
                let parameter_formats = parameters.iter()
                    .map(|parameter| ValueFormat::try_from(&parameter.value_type))
                    // This looks ugly but just collects the iterator of Result<T, E> into a Result<Vec<T>, E>
                    .collect::<Result<Vec<_>, _>>()?;

                // TODO: pretty much everything below this line is positively foul
                self.local_symbols.clear();
                self.next_anonymous_register_id = 1;
                self.next_anonymous_label_id = 1;
                let function_context = context.clone().enter_function();
                parameters.iter()
                    .map(|parameter| parameter.name.clone())
                    .zip(parameter_formats.iter())
                    .for_each(|(name, format)| {
                        let alignment = format.size();
                        let symbol = self.create_symbol(&function_context, name, format.clone(), alignment);
                        self.define_symbol(&function_context, symbol);
                    });

                let format = ValueFormat::Function {
                    returned,
                    parameters: parameter_formats, 
                    is_varargs: false, // TODO
                };
                let function_symbol = self.create_symbol(context, name.clone(), format, alignment);
                let parameter_registers: Vec<_> = parameters.iter()
                    .map(|parameter| self.local_symbols.find(&parameter.name).unwrap().register().clone())
                    .collect();

                llvm::emit_function_enter(&mut self.emitter, function_symbol.register(), &parameter_registers)
                    .map_err(|cause| self.file_error(cause))?;
                
                self.define_symbol(context, function_symbol);

                self.generate_node(body.as_ref(), &function_context, None)?;

                llvm::emit_function_exit(&mut self.emitter)
                    .map_err(|cause| self.file_error(cause))?;

                None
            },
            _ => return Err(self.error(String::from("unexpected node type")))
        };

        if let (Some(expected_format), Some(result_value)) = (&expected_format, &result) {
            self.enforce_format(result_value, expected_format)?;
        }

        Ok(result)
    }

    pub fn generate(mut self, parser: &mut ast::parse::Parser<'a, impl BufRead>) -> crate::Result<()> {
        llvm::emit_preamble(&mut self.emitter, self.filename)
            .map_err(|cause| self.file_error(cause))?;
        
        while let Some(statement) = parser.parse_statement(false, true)? {
            self.generate_node(statement.as_ref(), &ScopeContext::new(), None)?;
        }

        llvm::emit_postamble(&mut self.emitter)
            .map_err(|cause| self.file_error(cause))
    }
}
