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
        to_format: Box<ValueFormat>,
    },
    Array {
        item_format: Box<ValueFormat>,
        length: Option<usize>,
    },
    Function {
        is_defined: bool,
        return_format: Box<ValueFormat>,
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

    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Never => None,
            Self::Void => Some(0),
            Self::Boolean => Some(1),
            Self::Integer { size, .. } => Some(*size),
            Self::Pointer { .. } => Some(8),
            Self::Array { length: Some(length), item_format } => item_format.size().map(|item_size| item_size * length),
            Self::Array { length: None, .. } => None,
            Self::Function { .. } => None,
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, ValueFormat::Function { .. })
    }

    pub fn into_pointer(self) -> Self {
        Self::Pointer {
            to_format: Box::new(self),
        }
    }

    pub fn broadens_to(&self, other: &Self) -> bool {
        self == other || match (self, other) {
            (Self::Pointer { to_format: self_inner }, Self::Pointer { to_format: other_inner }) => {
                self_inner.broadens_to(other_inner.as_ref())
            },
            (Self::Array { item_format: self_item, length: _ }, Self::Array { item_format: other_item, length: None }) => {
                self_item.broadens_to(other_item.as_ref())
            },
            (Self::Array { item_format: self_item, length: Some(self_length) }, Self::Array { item_format: other_item, length: Some(other_length) }) => {
                self_length >= other_length && self_item.broadens_to(other_item.as_ref())
            },
            _ => false
        }
    }
}

impl TryFrom<&ast::ValueType> for ValueFormat {
    type Error = Box<dyn crate::Error>;

    fn try_from(value: &ast::ValueType) -> crate::Result<Self> {
        match value {
            ast::ValueType::Named(name) => {
                match name.as_str() {
                    "never" => Ok(Self::Never),
                    "void" => Ok(Self::Void),
                    "bool" => Ok(Self::Boolean),
                    "i8" => Ok(Self::Integer { size: 1, signed: true }),
                    "u8" => Ok(Self::Integer { size: 1, signed: false }),
                    "i16" => Ok(Self::Integer { size: 2, signed: true }),
                    "u16" => Ok(Self::Integer { size: 2, signed: false }),
                    "i32" => Ok(Self::Integer { size: 4, signed: true }),
                    "u32" => Ok(Self::Integer { size: 4, signed: false }),
                    "i64" => Ok(Self::Integer { size: 8, signed: true }),
                    "u64" => Ok(Self::Integer { size: 8, signed: false }),
                    _ => Err(crate::RawError::new(format!("unrecognized type name '{name}'")).into_boxed())
                }
            },
            ast::ValueType::Pointer(to_type) => {
                Self::try_from(to_type.as_ref()).map(Self::into_pointer)
            },
            ast::ValueType::Array(item_type, Some(length)) => {
                if let ast::Node::Literal(token::Literal::Integer(length)) = length.as_ref() {
                    Ok(Self::Array {
                        item_format: Box::new(Self::try_from(item_type.as_ref())?),
                        length: Some(*length as usize),
                    })
                }
                else {
                    Err(crate::RawError::new(String::from("array length must be constant")).into_boxed())
                }
            },
            ast::ValueType::Array(item_type, None) => {
                Ok(Self::Array {
                    item_format: Box::new(Self::try_from(item_type.as_ref())?),
                    length: None,
                })
            },
        }
    }
}

impl fmt::Display for ValueFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never | Self::Void => {
                write!(f, "void")
            },
            Self::Boolean => {
                write!(f, "i1")
            },
            Self::Integer { size, .. } => {
                write!(f, "i{bits}", bits = size * 8)
            },
            Self::Pointer { to_format } => {
                write!(f, "{to_format}*")
            },
            Self::Array { item_format, length: Some(length) } => {
                write!(f, "[{length} x {item_format}]")
            },
            Self::Array { item_format, length: None } => {
                write!(f, "{item_format}")
            },
            Self::Function { return_format, parameters, is_varargs, .. } => {
                write!(f, "{return_format}(")?;
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

    pub fn format_mut(&mut self) -> &mut ValueFormat {
        &mut self.format
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
pub enum Constant {
    ZeroInitializer(ValueFormat),
    Boolean(bool),
    Integer(u64, ValueFormat),
    String(token::StringValue),
    Array(Vec<Value>, ValueFormat),
    BitCast {
        value: Box<Value>,
        to_format: ValueFormat,
    },
    GetElementPointer {
        element_format: ValueFormat,
        aggregate_format: ValueFormat,
        pointer: Box<Value>,
        indices: Vec<Value>,
    },
}

impl Constant {
    pub fn format(&self) -> ValueFormat {
        match self {
            Self::ZeroInitializer(format) => format.clone(),
            Self::Boolean(_) => ValueFormat::Boolean,
            Self::Integer(_, format) => format.clone(),
            Self::String(value) => ValueFormat::Array {
                item_format: Box::new(ValueFormat::Integer { size: 1, signed: false }),
                length: Some(value.len()),
            },
            Self::Array(value, format) => ValueFormat::Array {
                item_format: Box::new(format.clone()),
                length: Some(value.len()),
            },
            Self::BitCast { to_format, .. } => to_format.clone(),
            Self::GetElementPointer { element_format, .. } => element_format.clone().into_pointer(),
        }
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroInitializer(_) => write!(f, "zeroinitializer"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Integer(value, _) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Array(value, _) => {
                write!(f, "[")?;
                let mut items = value.iter();
                if let Some(item) = items.next() {
                    write!(f, " {format} {item}", format = item.format())?;
                    for item in items {
                        write!(f, ", {format} {item}", format = item.format())?;
                    }
                    write!(f, " ")?;
                }
                write!(f, "]")
            },
            Self::BitCast { value, to_format } => {
                write!(f, "bitcast ({format} {value} to {to_format})", format = value.format())
            },
            Self::GetElementPointer { aggregate_format, pointer, indices, .. } => {
                write!(f, "getelementptr inbounds ({aggregate_format}, {pointer_format} {pointer}", pointer_format = pointer.format())?;
                for index in indices {
                    write!(f, ", {index_format} {index}", index_format = index.format())?;
                }
                write!(f, ")")
            },
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Never,
    Void,
    Constant(Constant),
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
            Self::Constant(constant) => constant.format(),
            Self::Register(register) => register.format().clone(),
            Self::Indirect { loaded_format, .. } => loaded_format.clone(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(constant) => write!(f, "{constant}"),
            Self::Register(register) => write!(f, "{register}"),
            _ => write!(f, "?")
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

fn error_unknown_compile_time_size(value_type: &ast::ValueType) -> Box<dyn crate::Error> {
    crate::RawError::new(format!("cannot use type '{value_type}' here, as it does not have a known size at compile time")).into_boxed()
}

pub struct Generator<W: Write> {
    emitter: llvm::Emitter<W>,
    next_anonymous_register_id: usize,
    next_anonymous_label_id: usize,
    next_anonymous_constant_id: usize,
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
            next_anonymous_constant_id: 0,
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
            name: format!(".label.{id}"),
        }
    }

    pub fn new_anonymous_constant(&mut self, format: ValueFormat) -> Register {
        let id = self.next_anonymous_constant_id;
        self.next_anonymous_constant_id += 1;

        Register {
            name: format!(".const.{id}"),
            format: format.into_pointer(),
            is_global: true,
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

    pub fn enforce_format(&self, value: Value, format: &ValueFormat) -> crate::Result<Value> {
        let got_format = value.format();

        if got_format.broadens_to(format) {
            Ok(value)
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
            self.emitter.emit_cmp_not_equal(&result, &value, &Value::Constant(Constant::Integer(0, from_format.clone())))?;
            
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
                        Value::Constant(Constant::Integer(*value, expected_format.clone().unwrap_or_else(ValueFormat::default_integer)))
                    },
                    token::Literal::Boolean(value) => {
                        Value::Constant(Constant::Boolean(*value))
                    },
                    token::Literal::String(value) => {
                        let constant = Constant::String(value.clone());
                        let constant_format = constant.format();

                        // I'm not satisfied with how hacky this is, but...
                        if let Some(ValueFormat::Pointer { to_format }) = &expected_format {
                            let pointer = self.new_anonymous_constant(constant_format.clone());

                            self.emitter.queue_anonymous_constant(&pointer, &constant);

                            if let ValueFormat::Array { length: None, .. } = to_format.as_ref() {
                                // Unsized arrays are problematic because they are actually *T pretending to be *[T]
                                // Again, this is... definitely a hack... and really disgusting
                                let integer_zero = Value::Constant(Constant::Integer(0, ValueFormat::default_integer()));
                                Value::Constant(Constant::GetElementPointer {
                                    element_format: ValueFormat::Array {
                                        item_format: Box::new(ValueFormat::Integer { size: 1, signed: false }),
                                        length: None,
                                    },
                                    aggregate_format: constant_format,
                                    pointer: Box::new(Value::Register(pointer)),
                                    indices: vec![integer_zero.clone(), integer_zero],
                                })
                            }
                            else {
                                Value::Register(pointer)
                            }
                        }
                        else {
                            Value::Constant(constant)
                        }
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
                        
                        if let ValueFormat::Pointer { to_format } = operand.format() {
                            Value::Indirect {
                                pointer: Box::new(operand),
                                loaded_format: *to_format,
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

                if let ValueFormat::Function { return_format, parameters, is_varargs, .. } = callee.format() {
                    let mut argument_values = Vec::new();

                    // Ensure that when arguments and parameter formats are zipped, all arguments are generated
                    // This is important for e.g. variadic arguments, which don't have corresponding parameters
                    let parameters_iter = parameters.iter()
                        .map(|parameter_format| Some(parameter_format))
                        .chain(std::iter::repeat(None));
                    for (argument, parameter_format) in arguments.iter().zip(parameters_iter) {
                        let argument = self.generate_node(argument.as_ref(), context, parameter_format.cloned())?;
                        let argument = self.coerce_to_rvalue(argument)?;

                        argument_values.push(argument);
                    }

                    if !is_varargs && arguments.len() > parameters.len() {
                        return Err(crate::RawError::new(format!("too many arguments (expected {}, got {})", parameters.len(), arguments.len())).into_boxed());
                    }
                    else if arguments.len() < parameters.len() {
                        return Err(crate::RawError::new(format!("too few arguments (expected {}, got {})", parameters.len(), arguments.len())).into_boxed());
                    }

                    match return_format.as_ref() {
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
                            let result = self.new_anonymous_register(*return_format);

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
            ast::Node::Let { name, value_type, value } => {
                let format = ValueFormat::try_from(value_type)?;
                if format.size().is_none() {
                    return Err(error_unknown_compile_time_size(value_type));
                }
                let (symbol, pointer) = self.get_symbol_table(context).create_indirect_symbol(name.clone(), format.clone());

                if context.is_global() {
                    // TODO: constant folding
                    let init_value = if let Some(node) = value {
                        if let Value::Constant(constant) = self.generate_node(node.as_ref(), context, Some(format.clone()))? {
                            constant
                        }
                        else {
                            return Err(crate::RawError::new(String::from("initial value for global must be constant")).into_boxed());
                        }
                    }
                    else {
                        Constant::ZeroInitializer(format.clone())
                    };

                    self.emitter.emit_global_allocation(&pointer, &init_value, false)?;
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
            ast::Node::Function { name, parameters, is_varargs, return_type, body: None } => {
                let return_format = ValueFormat::try_from(return_type)?;
                if return_format.size().is_none() {
                    return Err(error_unknown_compile_time_size(return_type));
                }
                let parameter_formats = parameters.iter()
                    .map(|parameter| ValueFormat::try_from(&parameter.value_type))
                    .collect::<Result<Vec<ValueFormat>, _>>()?;
                for (format, parameter) in std::iter::zip(&parameter_formats, parameters) {
                    if format.size().is_none() {
                        return Err(error_unknown_compile_time_size(&parameter.value_type));
                    }
                }

                let format = ValueFormat::Function {
                    is_defined: false,
                    return_format: Box::new(return_format.clone()),
                    parameters: parameter_formats.clone(), 
                    is_varargs: *is_varargs,
                };

                let function_register;
                if let Some(previous_symbol) = self.global_symbol_table.find(name) {
                    if let ValueFormat::Function {
                        return_format: previous_return_format,
                        parameters: previous_parameters,
                        is_varargs: previous_varargs,
                        ..
                    } = previous_symbol.format() {
                        if previous_return_format.as_ref() == &return_format && &previous_parameters == &parameter_formats && previous_varargs == *is_varargs {
                            if let Value::Register(register) = previous_symbol.value() {
                                function_register = register.clone();
                            }
                            else {
                                panic!("unexpected function value");
                            }
                        }
                        else {
                            return Err(crate::RawError::new(format!("conflicting signatures for function '{name}'")).into_boxed());
                        }
                    }
                    else {
                        return Err(crate::RawError::new(format!("function '{name}' conflicts with global variable of the same name")).into_boxed());
                    }
                }
                else {
                    let (symbol, register) = self.get_symbol_table(context).create_register_symbol(name.clone(), format);

                    self.define_symbol(context, symbol);
                    function_register = register;
                }

                self.emitter.queue_function_declaration(&function_register, &return_format, &parameter_formats, *is_varargs);

                Value::Void
            },
            ast::Node::Function { name, parameters, is_varargs, return_type, body: Some(body) } => {
                // TODO: functions need a *bit* of cleaning up...
                let return_format = ValueFormat::try_from(return_type)?;
                if return_format.size().is_none() {
                    return Err(error_unknown_compile_time_size(return_type));
                }
                let parameter_formats = parameters.iter()
                    .map(|parameter| ValueFormat::try_from(&parameter.value_type))
                    .collect::<Result<Vec<ValueFormat>, _>>()?;
                for (format, parameter) in std::iter::zip(&parameter_formats, parameters) {
                    if format.size().is_none() {
                        return Err(error_unknown_compile_time_size(&parameter.value_type));
                    }
                }

                self.local_symbol_table.clear();
                self.next_anonymous_register_id = 1;
                self.next_anonymous_label_id = 0;
                let function_context = context.clone().enter_function(return_format.clone());

                let parameter_handles: Vec<_> = parameters.iter()
                    .map(|parameter| parameter.name.clone())
                    .zip(parameter_formats.iter())
                    .map(|(name, format)| {
                        let input_register = Register {
                            name: format!(".arg.{name}"),
                            format: format.clone(),
                            is_global: false,
                        };
                        let (symbol, pointer) = self.get_symbol_table(&function_context).create_indirect_symbol(name, format.clone());
                        
                        (input_register, symbol, pointer)
                    })
                    .collect();

                let format = ValueFormat::Function {
                    is_defined: true,
                    return_format: Box::new(return_format.clone()),
                    parameters: parameter_formats.clone(), 
                    is_varargs: *is_varargs,
                };

                let function_register;
                if let Some(previous_symbol) = self.global_symbol_table.find_mut(name) {
                    if let ValueFormat::Function {
                        is_defined: already_defined,
                        return_format: previous_return_format,
                        parameters: previous_parameters,
                        is_varargs: previous_varargs,
                    } = previous_symbol.format() {
                        if previous_return_format.as_ref() == &return_format && &previous_parameters == &parameter_formats && previous_varargs == *is_varargs {
                            if already_defined {
                                return Err(crate::RawError::new(format!("multiple definition of function '{name}'")).into_boxed());
                            }
                            else if let Value::Register(register) = previous_symbol.value_mut() {
                                *register.format_mut() = format;
                                function_register = register.clone();
                            }
                            else {
                                panic!("unexpected function value");
                            }
                        }
                        else {
                            return Err(crate::RawError::new(format!("conflicting signatures for function '{name}'")).into_boxed());
                        }
                    }
                    else {
                        return Err(crate::RawError::new(format!("function '{name}' conflicts with global variable of the same name")).into_boxed());
                    }
                }
                else {
                    let (symbol, register) = self.get_symbol_table(context).create_register_symbol(name.clone(), format);

                    self.define_symbol(context, symbol);
                    function_register = register;
                }

                let input_registers: Vec<_> = parameter_handles.iter()
                    .map(|(register, _, _)| register.clone())
                    .collect();
                self.emitter.emit_function_enter(&function_register, &return_format, &input_registers, *is_varargs)?;

                for (input_register, symbol, pointer) in parameter_handles {
                    self.emitter.emit_local_allocation(&pointer, input_register.format())?;
                    self.emitter.emit_store(&Value::Register(input_register), &Value::Register(pointer))?;
                    self.define_symbol(&function_context, symbol);
                }

                let body_result = self.generate_node(body.as_ref(), &function_context, None)?;
                if &body_result != &Value::Never {
                    if let ValueFormat::Void = &return_format {
                        self.emitter.emit_return(None)?;
                    }
                    else {
                        return Err(crate::RawError::new(format!("non-void function '{name}' could finish without returning a value")).into_boxed());
                    }
                }

                self.emitter.emit_function_exit()?;

                Value::Void
            },
            _ => return Err(crate::RawError::new(String::from("unexpected node type")).into_boxed())
        };

        if let Some(expected_format) = &expected_format {
            self.enforce_format(result, expected_format)
        }
        else {
            Ok(result)
        }
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
