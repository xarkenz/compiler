pub mod info;
pub mod llvm;

use crate::token;
use crate::ast;
use crate::Error;

use std::io::{Write, BufRead};
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum ValueFormat {
    Never,
    Void,
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
    pub fn default_integer() -> Self {
        Self::Integer {
            size: 4,
            signed: true,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Never => 0,
            Self::Void => 0,
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
    type Error = Box<dyn crate::Error>;

    fn try_from(value: &ast::ValueType) -> crate::Result<Self> {
        match value {
            ast::ValueType::Named(name) => {
                match name.as_str() {
                    "never" => Ok(ValueFormat::Never),
                    "void" => Ok(ValueFormat::Void),
                    "bool" => Ok(ValueFormat::Boolean),
                    "i8" => Ok(ValueFormat::Integer { size: 1, signed: true }),
                    "u8" => Ok(ValueFormat::Integer { size: 1, signed: false }),
                    "i16" => Ok(ValueFormat::Integer { size: 2, signed: true }),
                    "u16" => Ok(ValueFormat::Integer { size: 2, signed: false }),
                    "i32" => Ok(ValueFormat::Integer { size: 4, signed: true }),
                    "u32" => Ok(ValueFormat::Integer { size: 4, signed: false }),
                    "i64" => Ok(ValueFormat::Integer { size: 8, signed: true }),
                    "u64" => Ok(ValueFormat::Integer { size: 8, signed: false }),
                    _ => Err(crate::RawError::new(format!("unrecognized type name 'name'")).into_boxed())
                }
            },
        }
    }
}

impl fmt::Display for ValueFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never => write!(f, "void"),
            Self::Void => write!(f, "void"),
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
    Never,
    Void,
    Boolean(bool),
    Integer(u64, ValueFormat),
    Register(Register),
}

impl RightValue {
    pub fn format(&self) -> ValueFormat {
        match self {
            Self::Never => ValueFormat::Never,
            Self::Void => ValueFormat::Void,
            Self::Boolean(_) => ValueFormat::Boolean,
            Self::Integer(_, format) => format.clone(),
            Self::Register(value) => value.format().clone(),
        }
    }
}

impl fmt::Display for RightValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never | Self::Void => write!(f, "?"),
            Self::Boolean(value) => value.fmt(f),
            Self::Integer(value, _) => value.fmt(f),
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
        ast::BinaryOperation::ShiftLeft => lhs_format,
        ast::BinaryOperation::ShiftRight => lhs_format,
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
    return_format: Option<ValueFormat>,
    break_label: Option<Label>,
    continue_label: Option<Label>,
}

impl ScopeContext {
    pub fn new() -> Self {
        Self {
            return_format: None,
            break_label: None,
            continue_label: None,
        }
    }

    pub fn is_global(&self) -> bool {
        self.return_format.is_none()
    }

    pub fn return_format(&self) -> Option<&ValueFormat> {
        self.return_format.as_ref()
    }

    pub fn break_label(&self) -> Option<&Label> {
        self.break_label.as_ref()
    }

    pub fn continue_label(&self) -> Option<&Label> {
        self.continue_label.as_ref()
    }

    pub fn enter_function(mut self, return_format: ValueFormat) -> Self {
        self.return_format = Some(return_format);

        self
    }

    pub fn enter_loop(mut self, break_label: Label, continue_label: Label) -> Self {
        self.break_label = Some(break_label);
        self.continue_label = Some(continue_label);

        self
    }
}

pub struct Generator<W: Write> {
    emitter: llvm::Emitter<W>,
    next_anonymous_register_id: usize,
    next_anonymous_label_id: usize,
    global_symbols: info::SymbolTable,
    local_symbols: info::SymbolTable,
}

impl Generator<std::fs::File> {
    pub fn from_filename(filename: String) -> crate::Result<Self> {
        llvm::Emitter::from_filename(filename)
            .map(|emitter| Self::new(emitter))
    }
}

impl<W: Write> Generator<W> {
    const DEFAULT_SYMBOL_TABLE_CAPACITY: usize = 256;

    pub fn new(emitter: llvm::Emitter<W>) -> Self {
        Self {
            emitter,
            next_anonymous_register_id: 1,
            next_anonymous_label_id: 0,
            global_symbols: info::SymbolTable::new(Self::DEFAULT_SYMBOL_TABLE_CAPACITY, true),
            local_symbols: info::SymbolTable::new(Self::DEFAULT_SYMBOL_TABLE_CAPACITY, false),
        }
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
            .ok_or_else(|| crate::RawError::new(format!("undefined symbol '{name}'")).into_boxed())
    }

    pub fn enforce_format(&self, value: &RightValue, format: &ValueFormat) -> crate::Result<()> {
        let got_format = value.format();

        if &got_format == format {
            Ok(())
        }
        else {
            Err(crate::RawError::new(format!("expected a value of type {format}, got {got_format} instead")).into_boxed())
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
                self.emitter.emit_extension(&result, &value)?;

                Ok(RightValue::Register(result))
            }
            else if to_size < from_size {
                let result = self.new_anonymous_register(to_format.clone());
                self.emitter.emit_truncation(&result, &value)?;

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
            self.emitter.emit_cmp_not_equal(&result, &value, &RightValue::Integer(0, from_format.clone()))?;
            
            Ok(RightValue::Register(result))
        }
        else if let (
            ValueFormat::Boolean,
            ValueFormat::Integer { .. },
        ) = (&from_format, to_format) {
            let result = self.new_anonymous_register(to_format.clone());
            self.emitter.emit_zero_extension(&result, &value)?;
            
            Ok(RightValue::Register(result))
        }
        else {
            Err(crate::RawError::new(format!("cannot convert from {from_format} to {to_format}")).into_boxed())
        }
    }

    pub fn generate_node(&mut self, node: &ast::Node, context: &ScopeContext, expected_format: Option<ValueFormat>) -> crate::Result<RightValue> {
        let result = match node {
            ast::Node::Literal(literal) => {
                match literal {
                    token::Literal::Identifier(name) => {
                        // If we don't clone here, with the way things are currently set up, we can't borrow *self as mutable
                        let symbol = self.get_symbol(name)?.clone();

                        if symbol.format().is_function() {
                            RightValue::Register(symbol.register().clone())
                        }
                        else {
                            let result = self.new_anonymous_register(symbol.format().clone());
                            self.emitter.emit_symbol_load(&result, &symbol)?;
                            RightValue::Register(result)
                        }
                    },
                    token::Literal::Integer(value) => {
                        RightValue::Integer(*value, expected_format.clone().unwrap_or(ValueFormat::Integer { size: 4, signed: true }))
                    },
                    token::Literal::Boolean(value) => {
                        RightValue::Boolean(*value)
                    }
                }
            },
            ast::Node::Unary { operation, operand } => {
                let expected_operand_format = expected_unary_operand_format(*operation, expected_format.clone());
                let operand = self.generate_node(operand.as_ref(), context, expected_operand_format)?;
                
                match operation {
                    ast::UnaryOperation::PostIncrement => todo!(),
                    ast::UnaryOperation::PostDecrement => todo!(),
                    ast::UnaryOperation::PreIncrement => todo!(),
                    ast::UnaryOperation::PreDecrement => todo!(),
                    ast::UnaryOperation::Positive => {
                        // Basically a no-op...
                        operand
                    },
                    ast::UnaryOperation::Negative => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(operand.format()));

                        self.emitter.emit_negation(&result, &operand)?;

                        RightValue::Register(result)
                    },
                    ast::UnaryOperation::BitwiseNot => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(operand.format()));

                        self.emitter.emit_inversion(&result, &operand)?;

                        RightValue::Register(result)
                    },
                    ast::UnaryOperation::LogicalNot => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        self.emitter.emit_inversion(&result, &operand)?;

                        RightValue::Register(result)
                    },
                    ast::UnaryOperation::Reference => todo!(),
                    ast::UnaryOperation::Dereference => todo!(),
                    ast::UnaryOperation::GetSize => todo!(),
                    ast::UnaryOperation::GetAlign => todo!(),
                }
            },
            ast::Node::Binary { operation: ast::BinaryOperation::Assign, lhs, rhs } => {
                if let ast::Node::Literal(token::Literal::Identifier(name)) = lhs.as_ref() {
                    // If we don't clone here, with the way things are currently set up, we can't borrow *self as mutable
                    let symbol = self.get_symbol(name)?.clone();
                    let value = self.generate_node(rhs.as_ref(), context, Some(symbol.format().clone()))?;

                    self.emitter.emit_symbol_store(&value, &symbol)?;

                    value
                }
                else {
                    return Err(crate::RawError::new(String::from("invalid left-hand side for '='")).into_boxed());
                }
            },
            ast::Node::Binary { operation: ast::BinaryOperation::Convert, lhs, rhs } => {
                if let ast::Node::ValueType(value_type) = rhs.as_ref() {
                    let to_format = ValueFormat::try_from(value_type)?;
                    let value = self.generate_node(lhs.as_ref(), context, None)?;

                    self.change_format(value, &to_format)?
                }
                else {
                    return Err(crate::RawError::new(String::from("invalid right-hand side for 'as'")).into_boxed());
                }
            }
            ast::Node::Binary { operation, lhs, rhs } => {
                let expected_lhs_format = expected_binary_lhs_format(*operation, expected_format.clone());
                let lhs = self.generate_node(lhs.as_ref(), context, expected_lhs_format)?;
                let expected_rhs_format = expected_binary_rhs_format(*operation, expected_format.clone(), Some(lhs.format()));
                let rhs = self.generate_node(rhs.as_ref(), context, expected_rhs_format)?;

                match operation {
                    ast::BinaryOperation::Subscript => todo!(),
                    ast::BinaryOperation::Access => todo!(),
                    ast::BinaryOperation::DerefAccess => todo!(),
                    ast::BinaryOperation::Convert => unreachable!(),
                    ast::BinaryOperation::Add => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_addition(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::Subtract => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_subtraction(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::Multiply => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_multiplication(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::Divide => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_division(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::Remainder => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_remainder(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::ShiftLeft => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_shift_left(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::ShiftRight => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_shift_right(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::BitwiseAnd => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_bitwise_and(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::BitwiseOr => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_bitwise_or(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::BitwiseXor => {
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(lhs.format()));

                        self.emitter.emit_bitwise_xor(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::Equal => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        self.emitter.emit_cmp_equal(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::NotEqual => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        self.emitter.emit_cmp_not_equal(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::LessThan => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        self.emitter.emit_cmp_less_than(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::LessEqual => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        self.emitter.emit_cmp_less_equal(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::GreaterThan => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        self.emitter.emit_cmp_greater_than(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::GreaterEqual => {
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        self.emitter.emit_cmp_greater_equal(&result, &lhs, &rhs)?;

                        RightValue::Register(result)
                    },
                    ast::BinaryOperation::LogicalAnd => todo!(),
                    ast::BinaryOperation::LogicalOr => todo!(),
                    ast::BinaryOperation::Assign => unreachable!(),
                    ast::BinaryOperation::MultiplyAssign => todo!(),
                    ast::BinaryOperation::DivideAssign => todo!(),
                    ast::BinaryOperation::RemainderAssign => todo!(),
                    ast::BinaryOperation::AddAssign => todo!(),
                    ast::BinaryOperation::SubtractAssign => todo!(),
                    ast::BinaryOperation::ShiftLeftAssign => todo!(),
                    ast::BinaryOperation::ShiftRightAssign => todo!(),
                    ast::BinaryOperation::BitwiseAndAssign => todo!(),
                    ast::BinaryOperation::BitwiseXorAssign => todo!(),
                    ast::BinaryOperation::BitwiseOrAssign => todo!(),
                }
            },
            ast::Node::Call { callee, arguments } => {
                let callee = self.generate_node(callee.as_ref(), context, None)?;

                if let ValueFormat::Function { returned, parameters, is_varargs } = callee.format() {
                    let mut argument_results = Vec::new();
                    for (argument, parameter_format) in arguments.iter().zip(parameters.iter()) {
                        let argument_result = self.generate_node(argument.as_ref(), context, Some(parameter_format.clone()))?;
                        argument_results.push(argument_result);
                    }

                    if !is_varargs && arguments.len() > parameters.len() {
                        return Err(crate::RawError::new(format!("too many arguments; expected {} arguments, got {}", parameters.len(), arguments.len())).into_boxed());
                    }
                    else if arguments.len() < parameters.len() {
                        return Err(crate::RawError::new(format!("too few arguments; expected {} arguments, got {}", parameters.len(), arguments.len())).into_boxed());
                    }

                    match returned.as_ref() {
                        ValueFormat::Never => {
                            self.emitter.emit_function_call(None, &callee, &argument_results)?;
                            self.emitter.emit_unreachable()?;

                            RightValue::Never
                        },
                        ValueFormat::Void => {
                            self.emitter.emit_function_call(None, &callee, &argument_results)?;

                            RightValue::Void
                        },
                        _ => {
                            let result = self.new_anonymous_register(*returned);

                            self.emitter.emit_function_call(Some(&result), &callee, &argument_results)?;

                            RightValue::Register(result)
                        }
                    }
                }
                else {
                    return Err(crate::RawError::new(String::from("cannot call a non-function object")).into_boxed());
                }
            },
            ast::Node::Scope { statements } => {
                self.local_symbols.enter_scope();

                let mut result = RightValue::Void;
                for statement in statements {
                    let statement_result = self.generate_node(statement.as_ref(), context, None)?;
                    if let RightValue::Never = statement_result {
                        // The rest of the statements in the block will never be executed, so they don't need to be generated
                        result = RightValue::Never;
                        break;
                    }
                }

                self.local_symbols.leave_scope();

                result
            },
            ast::Node::Conditional { condition, consequent, alternative } => {
                let condition = self.generate_node(condition.as_ref(), context, Some(ValueFormat::Boolean))?;
                let consequent_label = self.new_anonymous_label();
                let alternative_label = self.new_anonymous_label();

                self.emitter.emit_conditional_branch(&condition, &consequent_label, &alternative_label)?;
                
                if let Some(alternative) = alternative {
                    let tail_label = self.new_anonymous_label();

                    self.emitter.emit_label(&consequent_label)?;
                    let consequent_result = self.generate_node(consequent.as_ref(), context, None)?;
                    self.emitter.emit_unconditional_branch(&tail_label)?;

                    self.emitter.emit_label(&alternative_label)?;
                    let alternative_result = self.generate_node(alternative.as_ref(), context, None)?;
                    self.emitter.emit_unconditional_branch(&tail_label)?;

                    self.emitter.emit_label(&tail_label)?;

                    if let (RightValue::Never, RightValue::Never) = (consequent_result, alternative_result) {
                        RightValue::Never
                    }
                    else {
                        RightValue::Void
                    }
                }
                else {
                    self.emitter.emit_label(&consequent_label)?;
                    self.generate_node(consequent.as_ref(), context, None)?;
                    self.emitter.emit_unconditional_branch(&alternative_label)?;

                    self.emitter.emit_label(&alternative_label)?;

                    RightValue::Void
                }
            },
            ast::Node::While { condition, body } => {
                // TODO: handling never, break/continue vs. return
                let condition_label = self.new_anonymous_label();

                self.emitter.emit_unconditional_branch(&condition_label)?;

                self.emitter.emit_label(&condition_label)?;

                let condition = self.generate_node(condition.as_ref(), context, Some(ValueFormat::Boolean))?;
                let body_label = self.new_anonymous_label();
                let tail_label = self.new_anonymous_label();
                let loop_context = context.clone().enter_loop(tail_label.clone(), condition_label.clone());

                self.emitter.emit_conditional_branch(&condition, &body_label, &tail_label)?;

                self.emitter.emit_label(&body_label)?;
                self.generate_node(body.as_ref(), &loop_context, None)?;
                self.emitter.emit_unconditional_branch(&condition_label)?;

                self.emitter.emit_label(&tail_label)?;

                RightValue::Void
            },
            ast::Node::Break => {
                let break_label = context.break_label()
                    .ok_or_else(|| crate::RawError::new(String::from("unexpected 'break' outside loop")).into_boxed())?;

                self.emitter.emit_unconditional_branch(break_label)?;

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                self.next_anonymous_register_id += 1;

                RightValue::Never
            },
            ast::Node::Continue => {
                let continue_label = context.continue_label()
                    .ok_or_else(|| crate::RawError::new(String::from("unexpected 'continue' outside loop")).into_boxed())?;

                self.emitter.emit_unconditional_branch(continue_label)?;

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                self.next_anonymous_register_id += 1;

                RightValue::Never
            },
            ast::Node::Return { value } => {
                let return_format = context.return_format()
                    .ok_or_else(|| crate::RawError::new(String::from("'return' outside of function")).into_boxed())?;

                if let Some(value) = value {
                    if let ValueFormat::Void = return_format {
                        return Err(crate::RawError::new(String::from("returning a non-void value from a void function")).into_boxed());
                    }
                    else {
                        let value = self.generate_node(value.as_ref(), context, Some(return_format.clone()))?;
                        self.emitter.emit_return(Some(&value))?;
                    }
                }
                else {
                    if let ValueFormat::Void = return_format {
                        self.emitter.emit_return(None)?;
                    }
                    else {
                        return Err(crate::RawError::new(String::from("returning a non-void value from a void function")).into_boxed());
                    }
                }

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                self.next_anonymous_register_id += 1;

                RightValue::Never
            },
            ast::Node::Print { value } => {
                let value_to_print = self.generate_node(value.as_ref(), context, None)?;
                let to_format = match value_to_print.format() {
                    ValueFormat::Boolean => ValueFormat::Integer { size: 8, signed: false },
                    ValueFormat::Integer { signed, .. } => ValueFormat::Integer { size: 8, signed },
                    format => format
                };
                let value_to_print = self.change_format(value_to_print, &to_format)?;
                let result_register = self.new_anonymous_register(ValueFormat::Integer { size: 4, signed: true });

                self.emitter.emit_print(&result_register, &value_to_print)?;

                RightValue::Void
            },
            ast::Node::Let { name, value_type, value } => {
                let format = ValueFormat::try_from(value_type)?;
                let alignment = format.size();
                let symbol = self.create_symbol(context, name.clone(), format.clone(), alignment);

                self.emitter.emit_symbol_allocation(&symbol)?;

                if let Some(node) = value {
                    let value = self.generate_node(node.as_ref(), context, Some(format))?;

                    self.emitter.emit_symbol_store(&value, &symbol)?;
                }

                self.define_symbol(context, symbol);

                RightValue::Void
            },
            ast::Node::Function { name, parameters, return_type, body } => {
                let returned = ValueFormat::try_from(return_type)?;
                let alignment = returned.size(); // TODO: what would this even mean
                let parameter_formats = parameters.iter()
                    .map(|parameter| ValueFormat::try_from(&parameter.value_type))
                    // This looks ugly but just collects the iterator of Result<T, E> into a Result<Vec<T>, E>
                    .collect::<Result<Vec<_>, _>>()?;

                // TODO: pretty much everything below this line is positively foul
                self.local_symbols.clear();
                self.next_anonymous_register_id = 1;
                self.next_anonymous_label_id = 0;
                let function_context = context.clone().enter_function(returned.clone());

                let parameter_handles: Vec<_> = parameters.iter()
                    .map(|parameter| parameter.name.clone())
                    .zip(parameter_formats.iter())
                    .map(|(name, format)| {
                        let alignment = format.size();
                        let parameter_register = Register {
                            name: format!("-arg-{name}"),
                            format: format.clone(),
                            is_global: false,
                        };
                        let parameter_symbol = self.create_symbol(&function_context, name, format.clone(), alignment);
                        (parameter_symbol, parameter_register)
                    })
                    .collect();

                let format = ValueFormat::Function {
                    returned: Box::new(returned.clone()),
                    parameters: parameter_formats, 
                    is_varargs: false, // TODO
                };
                let function_symbol = self.create_symbol(context, name.clone(), format, alignment);

                self.emitter.emit_function_enter(function_symbol.register(), &parameter_handles)?;
                self.define_symbol(context, function_symbol);

                for (parameter_symbol, parameter_register) in parameter_handles {
                    self.emitter.emit_symbol_allocation(&parameter_symbol)?;
                    self.emitter.emit_symbol_store(&RightValue::Register(parameter_register), &parameter_symbol)?;
                    self.define_symbol(&function_context, parameter_symbol);
                }

                let body_result = self.generate_node(body.as_ref(), &function_context, None)?;
                if !(matches!(body_result, RightValue::Never)) {
                    if let ValueFormat::Void = &returned {
                        self.emitter.emit_return(None)?;
                    }
                    else {
                        return Err(crate::RawError::new(String::from("non-void function could finish without returning a value")).into_boxed());
                    }
                }

                self.emitter.emit_function_exit()?;

                RightValue::Void
            },
            _ => return Err(crate::RawError::new(String::from("unexpected node type")).into_boxed())
        };

        if let Some(expected_format) = &expected_format {
            self.enforce_format(&result, expected_format)?;
        }

        Ok(result)
    }

    pub fn generate<T: BufRead>(mut self, parser: &mut ast::parse::Parser<T>) -> crate::Result<()> {
        self.emitter.emit_preamble(parser.filename())?;
        
        let global_context = ScopeContext::new();

        while let Some(statement) = parser.parse_statement(false, true)? {
            self.generate_node(statement.as_ref(), &global_context, None)?;
        }

        self.emitter.emit_postamble()
    }
}
