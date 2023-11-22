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
                    _ => Err(crate::RawError::new(format!("unrecognized type name '{name}'")).into_boxed())
                }
            },
            ast::ValueType::Pointer(to_type) => {
                Self::try_from(to_type.as_ref()).map(Self::into_pointer)
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
pub enum Value {
    Never,
    Void,
    Boolean(bool),
    Integer(u64, ValueFormat),
    Register(Register),
    Indirect {
        pointer: Box<Value>,
        loaded_format: ValueFormat,
    },
}

impl Value {
    pub fn format(&self) -> ValueFormat {
        match self {
            Self::Never => ValueFormat::Never,
            Self::Void => ValueFormat::Void,
            Self::Boolean(_) => ValueFormat::Boolean,
            Self::Integer(_, format) => format.clone(),
            Self::Register(register) => register.format().clone(),
            Self::Indirect { loaded_format, .. } => loaded_format.clone(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never | Self::Void => write!(f, "?"),
            Self::Boolean(value) => value.fmt(f),
            Self::Integer(value, _) => value.fmt(f),
            Self::Register(register) => register.fmt(f),
            Self::Indirect { .. } => write!(f, "?"),
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
    global_symbol_table: info::SymbolTable,
    local_symbol_table: info::SymbolTable,
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
            global_symbol_table: info::SymbolTable::new(Self::DEFAULT_SYMBOL_TABLE_CAPACITY, true),
            local_symbol_table: info::SymbolTable::new(Self::DEFAULT_SYMBOL_TABLE_CAPACITY, false),
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

    pub fn get_symbol_table(&self, context: &ScopeContext) -> &info::SymbolTable {
        if context.is_global() {
            &self.global_symbol_table
        }
        else {
            &self.local_symbol_table
        }
    }

    pub fn define_symbol(&mut self, context: &ScopeContext, symbol: info::Symbol) {
        if context.is_global() {
            self.global_symbol_table.insert(symbol);
        }
        else {
            self.local_symbol_table.insert(symbol);
        }
    }

    pub fn get_symbol(&self, name: &str) -> crate::Result<&info::Symbol> {
        self.local_symbol_table.find(name)
            .or_else(|| self.global_symbol_table.find(name))
            .ok_or_else(|| crate::RawError::new(format!("undefined symbol '{name}'")).into_boxed())
    }

    pub fn enforce_format(&self, value: &Value, format: &ValueFormat) -> crate::Result<()> {
        let got_format = value.format();

        if &got_format == format {
            Ok(())
        }
        else {
            Err(crate::RawError::new(format!("expected a value of type {format}, got {got_format} instead")).into_boxed())
        }
    }

    pub fn change_format(&mut self, value: Value, to_format: &ValueFormat) -> crate::Result<Value> {
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

                Ok(Value::Register(result))
            }
            else if to_size < from_size {
                let result = self.new_anonymous_register(to_format.clone());
                self.emitter.emit_truncation(&result, &value)?;

                Ok(Value::Register(result))
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
            self.emitter.emit_cmp_not_equal(&result, &value, &Value::Integer(0, from_format.clone()))?;
            
            Ok(Value::Register(result))
        }
        else if let (
            ValueFormat::Boolean,
            ValueFormat::Integer { .. },
        ) = (&from_format, to_format) {
            let result = self.new_anonymous_register(to_format.clone());
            self.emitter.emit_zero_extension(&result, &value)?;
            
            Ok(Value::Register(result))
        }
        else {
            Err(crate::RawError::new(format!("cannot convert from '{from_format}' to '{to_format}'")).into_boxed())
        }
    }

    pub fn coerce_to_rvalue(&mut self, value: Value) -> crate::Result<Value> {
        if let Value::Indirect { pointer, loaded_format } = value {
            let result = self.new_anonymous_register(loaded_format);
            self.emitter.emit_load(&result, pointer.as_ref())?;
            
            Ok(Value::Register(result))
        }
        else {
            Ok(value)
        }
    }

    pub fn generate_binary_arithmetic_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<ValueFormat>) -> crate::Result<(Register, Value, Value)> {
        let lhs = self.generate_node(lhs, context, expected_format.clone())?;
        let lhs = self.coerce_to_rvalue(lhs)?;

        let rhs = self.generate_node(rhs, context, Some(lhs.format()))?;
        let rhs = self.coerce_to_rvalue(rhs)?;

        let result = self.new_anonymous_register(expected_format.unwrap_or_else(|| lhs.format()));

        Ok((result, lhs, rhs))
    }

    pub fn generate_comparison_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext) -> crate::Result<(Register, Value, Value)> {
        let lhs = self.generate_node(lhs, context, None)?;
        let lhs = self.coerce_to_rvalue(lhs)?;

        let rhs = self.generate_node(rhs, context, Some(lhs.format()))?;
        let rhs = self.coerce_to_rvalue(rhs)?;

        let result = self.new_anonymous_register(ValueFormat::Boolean);

        Ok((result, lhs, rhs))
    }

    pub fn generate_assignment(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<ValueFormat>) -> crate::Result<Value> {
        let lhs = self.generate_node(lhs, context, expected_format.clone())?;

        if let Value::Indirect { pointer, loaded_format } = lhs {
            let rhs = self.generate_node(rhs, context, Some(loaded_format))?;
            let rhs = self.coerce_to_rvalue(rhs)?;

            self.emitter.emit_store(&rhs, pointer.as_ref())?;

            Ok(rhs)
        }
        else {
            Err(crate::RawError::new(String::from("left-hand side of '=' must be an lvalue")).into_boxed())
        }
    }

    pub fn generate_node(&mut self, node: &ast::Node, context: &ScopeContext, expected_format: Option<ValueFormat>) -> crate::Result<Value> {
        let result = match node {
            ast::Node::Literal(literal) => {
                match literal {
                    token::Literal::Identifier(name) => {
                        self.get_symbol(name)?.value().clone()
                    },
                    token::Literal::Integer(value) => {
                        Value::Integer(*value, expected_format.clone().unwrap_or_else(ValueFormat::default_integer))
                    },
                    token::Literal::Boolean(value) => {
                        Value::Boolean(*value)
                    }
                }
            },
            ast::Node::Unary { operation, operand } => {
                match operation {
                    ast::UnaryOperation::PostIncrement => todo!(),
                    ast::UnaryOperation::PostDecrement => todo!(),
                    ast::UnaryOperation::PreIncrement => todo!(),
                    ast::UnaryOperation::PreDecrement => todo!(),
                    ast::UnaryOperation::Positive => {
                        let operand = self.generate_node(operand.as_ref(), context, expected_format.clone())?;

                        self.coerce_to_rvalue(operand)?
                    },
                    ast::UnaryOperation::Negative => {
                        let operand = self.generate_node(operand.as_ref(), context, expected_format.clone())?;
                        let operand = self.coerce_to_rvalue(operand)?;
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(operand.format()));

                        self.emitter.emit_negation(&result, &operand)?;

                        Value::Register(result)
                    },
                    ast::UnaryOperation::BitwiseNot => {
                        let operand = self.generate_node(operand.as_ref(), context, expected_format.clone())?;
                        let operand = self.coerce_to_rvalue(operand)?;
                        let result = self.new_anonymous_register(expected_format.clone().unwrap_or(operand.format()));

                        self.emitter.emit_inversion(&result, &operand)?;

                        Value::Register(result)
                    },
                    ast::UnaryOperation::LogicalNot => {
                        let operand = self.generate_node(operand.as_ref(), context, Some(ValueFormat::Boolean))?;
                        let operand = self.coerce_to_rvalue(operand)?;
                        let result = self.new_anonymous_register(ValueFormat::Boolean);

                        self.emitter.emit_inversion(&result, &operand)?;

                        Value::Register(result)
                    },
                    ast::UnaryOperation::Reference => {
                        let operand = self.generate_node(operand.as_ref(), context, None)?;

                        if let Value::Indirect { pointer, .. } = operand {
                            *pointer
                        }
                        else {
                            return Err(crate::RawError::new(String::from("operand of '&' must be an lvalue")).into_boxed());
                        }
                    },
                    ast::UnaryOperation::Dereference => {
                        let operand = self.generate_node(operand.as_ref(), context, expected_format.clone().map(ValueFormat::into_pointer))?;
                        let operand = self.coerce_to_rvalue(operand)?;
                        
                        if let ValueFormat::Pointer { to } = operand.format() {
                            Value::Indirect {
                                pointer: Box::new(operand),
                                loaded_format: *to,
                            }
                        }
                        else {
                            return Err(crate::RawError::new(format!("cannot dereference value of type '{}'", operand.format())).into_boxed());
                        }
                    },
                    ast::UnaryOperation::GetSize => todo!(),
                    ast::UnaryOperation::GetAlign => todo!(),
                }
            },
            ast::Node::Binary { operation, lhs, rhs } => {
                match operation {
                    ast::BinaryOperation::Subscript => todo!(),
                    ast::BinaryOperation::Access => todo!(),
                    ast::BinaryOperation::DerefAccess => todo!(),
                    ast::BinaryOperation::Convert => {
                        if let ast::Node::ValueType(value_type) = rhs.as_ref() {
                            let value = self.generate_node(lhs.as_ref(), context, None)?;
                            let value = self.coerce_to_rvalue(value)?;
                            
                            let to_format = ValueFormat::try_from(value_type)?;
        
                            self.change_format(value, &to_format)?
                        }
                        else {
                            return Err(crate::RawError::new(String::from("invalid right-hand side for 'as'")).into_boxed());
                        }
                    },
                    ast::BinaryOperation::Add => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_addition(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::Subtract => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_subtraction(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::Multiply => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_multiplication(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::Divide => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_division(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::Remainder => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_remainder(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::ShiftLeft => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_shift_left(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::ShiftRight => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_shift_right(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::BitwiseAnd => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_bitwise_and(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::BitwiseOr => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_bitwise_or(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::BitwiseXor => {
                        let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?;

                        self.emitter.emit_bitwise_xor(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::Equal => {
                        let (result, lhs, rhs) = self.generate_comparison_operands(lhs.as_ref(), rhs.as_ref(), context)?;

                        self.emitter.emit_cmp_equal(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::NotEqual => {
                        let (result, lhs, rhs) = self.generate_comparison_operands(lhs.as_ref(), rhs.as_ref(), context)?;

                        self.emitter.emit_cmp_not_equal(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::LessThan => {
                        let (result, lhs, rhs) = self.generate_comparison_operands(lhs.as_ref(), rhs.as_ref(), context)?;

                        self.emitter.emit_cmp_less_than(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::LessEqual => {
                        let (result, lhs, rhs) = self.generate_comparison_operands(lhs.as_ref(), rhs.as_ref(), context)?;

                        self.emitter.emit_cmp_less_equal(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::GreaterThan => {
                        let (result, lhs, rhs) = self.generate_comparison_operands(lhs.as_ref(), rhs.as_ref(), context)?;

                        self.emitter.emit_cmp_greater_than(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::GreaterEqual => {
                        let (result, lhs, rhs) = self.generate_comparison_operands(lhs.as_ref(), rhs.as_ref(), context)?;

                        self.emitter.emit_cmp_greater_equal(&result, &lhs, &rhs)?;

                        Value::Register(result)
                    },
                    ast::BinaryOperation::LogicalAnd => todo!(),
                    ast::BinaryOperation::LogicalOr => todo!(),
                    ast::BinaryOperation::Assign => {
                        self.generate_assignment(lhs.as_ref(), rhs.as_ref(), context, expected_format.clone())?
                    },
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
                let callee = self.coerce_to_rvalue(callee)?;

                if let ValueFormat::Function { returned, parameters, is_varargs } = callee.format() {
                    let mut argument_values = Vec::new();
                    for (argument, parameter_format) in arguments.iter().zip(parameters.iter()) {
                        let argument = self.generate_node(argument.as_ref(), context, Some(parameter_format.clone()))?;
                        let argument = self.coerce_to_rvalue(argument)?;

                        argument_values.push(argument);
                    }

                    if !is_varargs && arguments.len() > parameters.len() {
                        return Err(crate::RawError::new(format!("too many arguments (expected {}, got {})", parameters.len(), arguments.len())).into_boxed());
                    }
                    else if arguments.len() < parameters.len() {
                        return Err(crate::RawError::new(format!("too few arguments (expected {}, got {})", parameters.len(), arguments.len())).into_boxed());
                    }

                    match returned.as_ref() {
                        ValueFormat::Never => {
                            self.emitter.emit_function_call(None, &callee, &argument_values)?;
                            self.emitter.emit_unreachable()?;

                            Value::Never
                        },
                        ValueFormat::Void => {
                            self.emitter.emit_function_call(None, &callee, &argument_values)?;

                            Value::Void
                        },
                        _ => {
                            let result = self.new_anonymous_register(*returned);

                            self.emitter.emit_function_call(Some(&result), &callee, &argument_values)?;

                            Value::Register(result)
                        }
                    }
                }
                else {
                    return Err(crate::RawError::new(String::from("cannot call a non-function object")).into_boxed());
                }
            },
            ast::Node::Scope { statements } => {
                self.local_symbol_table.enter_scope();

                let mut result = Value::Void;
                for statement in statements {
                    let statement_value = self.generate_node(statement.as_ref(), context, None)?;
                    if let Value::Never = statement_value {
                        // The rest of the statements in the block will never be executed, so they don't need to be generated
                        result = Value::Never;
                        break;
                    }
                }

                self.local_symbol_table.leave_scope();

                result
            },
            ast::Node::Conditional { condition, consequent, alternative } => {
                let condition = self.generate_node(condition.as_ref(), context, Some(ValueFormat::Boolean))?;
                let condition = self.coerce_to_rvalue(condition)?;

                let consequent_label = self.new_anonymous_label();
                let alternative_label = self.new_anonymous_label();

                self.emitter.emit_conditional_branch(&condition, &consequent_label, &alternative_label)?;
                
                if let Some(alternative) = alternative {
                    let tail_label = self.new_anonymous_label();

                    self.emitter.emit_label(&consequent_label)?;
                    let consequent_value = self.generate_node(consequent.as_ref(), context, None)?;
                    self.emitter.emit_unconditional_branch(&tail_label)?;

                    self.emitter.emit_label(&alternative_label)?;
                    let alternative_value = self.generate_node(alternative.as_ref(), context, None)?;
                    self.emitter.emit_unconditional_branch(&tail_label)?;

                    self.emitter.emit_label(&tail_label)?;

                    if let (Value::Never, Value::Never) = (consequent_value, alternative_value) {
                        Value::Never
                    }
                    else {
                        Value::Void
                    }
                }
                else {
                    self.emitter.emit_label(&consequent_label)?;
                    self.generate_node(consequent.as_ref(), context, None)?;
                    self.emitter.emit_unconditional_branch(&alternative_label)?;

                    self.emitter.emit_label(&alternative_label)?;

                    Value::Void
                }
            },
            ast::Node::While { condition, body } => {
                // TODO: handling never, break/continue vs. return
                let condition_label = self.new_anonymous_label();

                self.emitter.emit_unconditional_branch(&condition_label)?;

                self.emitter.emit_label(&condition_label)?;

                let condition = self.generate_node(condition.as_ref(), context, Some(ValueFormat::Boolean))?;
                let condition = self.coerce_to_rvalue(condition)?;

                let body_label = self.new_anonymous_label();
                let tail_label = self.new_anonymous_label();
                let loop_context = context.clone().enter_loop(tail_label.clone(), condition_label.clone());

                self.emitter.emit_conditional_branch(&condition, &body_label, &tail_label)?;

                self.emitter.emit_label(&body_label)?;
                self.generate_node(body.as_ref(), &loop_context, None)?;
                self.emitter.emit_unconditional_branch(&condition_label)?;

                self.emitter.emit_label(&tail_label)?;

                Value::Void
            },
            ast::Node::Break => {
                let break_label = context.break_label()
                    .ok_or_else(|| crate::RawError::new(String::from("unexpected 'break' outside loop")).into_boxed())?;

                self.emitter.emit_unconditional_branch(break_label)?;

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                self.next_anonymous_register_id += 1;

                Value::Never
            },
            ast::Node::Continue => {
                let continue_label = context.continue_label()
                    .ok_or_else(|| crate::RawError::new(String::from("unexpected 'continue' outside loop")).into_boxed())?;

                self.emitter.emit_unconditional_branch(continue_label)?;

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                self.next_anonymous_register_id += 1;

                Value::Never
            },
            ast::Node::Return { value } => {
                let return_format = context.return_format()
                    .ok_or_else(|| crate::RawError::new(String::from("'return' outside of function")).into_boxed())?;

                if let Some(value) = value {
                    if let ValueFormat::Void = return_format {
                        return Err(crate::RawError::new(String::from("returning without a value from a non-void function")).into_boxed());
                    }
                    else {
                        let value = self.generate_node(value.as_ref(), context, Some(return_format.clone()))?;
                        let value = self.coerce_to_rvalue(value)?;

                        self.emitter.emit_return(Some(&value))?;
                    }
                }
                else {
                    if let ValueFormat::Void = return_format {
                        self.emitter.emit_return(None)?;
                    }
                    else {
                        return Err(crate::RawError::new(String::from("returning with a value from a void function")).into_boxed());
                    }
                }

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                self.next_anonymous_register_id += 1;

                Value::Never
            },
            ast::Node::Print { value } => {
                let value_to_print = self.generate_node(value.as_ref(), context, None)?;
                let value_to_print = self.coerce_to_rvalue(value_to_print)?;
                let to_format = match value_to_print.format() {
                    ValueFormat::Boolean => ValueFormat::Integer { size: 8, signed: false },
                    ValueFormat::Integer { signed, .. } => ValueFormat::Integer { size: 8, signed },
                    format => format
                };
                let value_to_print = self.change_format(value_to_print, &to_format)?;
                let result_register = self.new_anonymous_register(ValueFormat::Integer { size: 4, signed: true });

                self.emitter.emit_print(&result_register, &value_to_print)?;

                Value::Void
            },
            ast::Node::Let { name, value_type, value } => {
                let format = ValueFormat::try_from(value_type)?;
                let (symbol, pointer) = self.get_symbol_table(context).create_indirect_symbol(name.clone(), format.clone());

                if context.is_global() {
                    // TODO: allow a constant for initializing global
                    if value.is_some() {
                        return Err(crate::RawError::new(String::from("initializing globals is not yet supported")).into_boxed());
                    }
                    self.emitter.emit_global_allocation(&pointer, &Value::Integer(0, format.clone()))?;
                }
                else {
                    self.emitter.emit_local_allocation(&pointer, &format)?;

                    if let Some(node) = value {
                        let value = self.generate_node(node.as_ref(), context, Some(format.clone()))?;
                        let value = self.coerce_to_rvalue(value)?;

                        self.emitter.emit_store(&value, &Value::Register(pointer))?;
                    }
                }

                self.define_symbol(context, symbol);

                Value::Void
            },
            ast::Node::Function { name, parameters, return_type, body } => {
                let returned = ValueFormat::try_from(return_type)?;
                let parameter_formats = parameters.iter()
                    .map(|parameter| ValueFormat::try_from(&parameter.value_type))
                    // This looks ugly but just collects the iterator of Result<T, E> into a Result<Vec<T>, E>
                    .collect::<Result<Vec<_>, _>>()?;

                // TODO: pretty much everything below this line is positively foul
                self.local_symbol_table.clear();
                self.next_anonymous_register_id = 1;
                self.next_anonymous_label_id = 0;
                let function_context = context.clone().enter_function(returned.clone());

                let parameter_handles: Vec<_> = parameters.iter()
                    .map(|parameter| parameter.name.clone())
                    .zip(parameter_formats.iter())
                    .map(|(name, format)| {
                        let input_register = Register {
                            name: format!("-arg-{name}"),
                            format: format.clone(),
                            is_global: false,
                        };
                        let (symbol, pointer) = self.get_symbol_table(&function_context).create_indirect_symbol(name, format.clone());
                        
                        (input_register, symbol, pointer)
                    })
                    .collect();

                let format = ValueFormat::Function {
                    returned: Box::new(returned.clone()),
                    parameters: parameter_formats, 
                    is_varargs: false, // TODO
                };
                let (function_symbol, function_register) = self.get_symbol_table(context).create_register_symbol(name.clone(), format);

                let input_registers: Vec<_> = parameter_handles.iter()
                    .map(|(register, _, _)| register.clone())
                    .collect();
                self.emitter.emit_function_enter(&function_register, &input_registers)?;
                self.define_symbol(context, function_symbol);

                for (input_register, symbol, pointer) in parameter_handles {
                    self.emitter.emit_local_allocation(&pointer, input_register.format())?;
                    self.emitter.emit_store(&Value::Register(input_register), &Value::Register(pointer))?;
                    self.define_symbol(&function_context, symbol);
                }

                let body_result = self.generate_node(body.as_ref(), &function_context, None)?;
                if &body_result != &Value::Never {
                    if let ValueFormat::Void = &returned {
                        self.emitter.emit_return(None)?;
                    }
                    else {
                        return Err(crate::RawError::new(String::from("non-void function could finish without returning a value")).into_boxed());
                    }
                }

                self.emitter.emit_function_exit()?;

                Value::Void
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
