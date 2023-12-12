pub mod info;
pub mod llvm;

use crate::token;
use crate::ast;
use crate::Error;

use std::io::{Write, BufRead};
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum Format {
    Never,
    Void,
    Mutable(Box<Format>),
    Boolean,
    Integer {
        size: usize,
        signed: bool,
    },
    Pointer(Box<Format>),
    Array {
        item_format: Box<Format>,
        length: Option<usize>,
    },
    Function {
        is_defined: bool,
        return_format: Box<Format>,
        parameters: Vec<Format>,
        is_varargs: bool,
    },
}

impl Format {
    pub fn opaque_pointer() -> Self {
        Self::Pointer(Box::new(Format::Void))
    }

    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Never => None,
            Self::Void => Some(0),
            Self::Mutable(format) => format.size(),
            Self::Boolean => Some(1),
            Self::Integer { size, .. } => Some(*size),
            Self::Pointer(_) => Some(8),
            Self::Array { length: Some(length), item_format } => item_format.size().map(|item_size| item_size * length),
            Self::Array { length: None, .. } => None,
            Self::Function { .. } => None,
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Format::Function { .. })
    }

    pub fn into_mutable(self) -> Self {
        Self::Mutable(Box::new(self))
    }

    pub fn into_pointer(self) -> Self {
        Self::Pointer(Box::new(self))
    }

    pub fn as_unqualified(&self) -> &Format {
        match self {
            Self::Mutable(format) => format.as_ref(),
            _ => self
        }
    }

    pub fn can_coerce_to(&self, other: &Self, is_concrete: bool) -> bool {
        self == other || match (self, other) {
            (Self::Mutable(self_format), Self::Mutable(other_format)) => {
                self_format.can_coerce_to(other_format.as_ref(), is_concrete)
            },
            (Self::Mutable(self_format), other_format) => {
                self_format.can_coerce_to(other_format, is_concrete)
            },
            (self_format, Self::Mutable(other_format)) => {
                // For example, `let x: i32 = 0; let y: mut i32 = x;` is fine, but `let x: *i32 = null; let y: *mut i32 = x;` is not
                is_concrete && self_format.can_coerce_to(other_format.as_ref(), true)
            },
            (Self::Pointer(self_pointee), Self::Pointer(other_pointee)) => {
                self_pointee.can_coerce_to(other_pointee.as_ref(), false)
            },
            (Self::Array { item_format: self_item, length: _ }, Self::Array { item_format: other_item, length: None }) => {
                !is_concrete && self_item.can_coerce_to(other_item.as_ref(), false)
            },
            (Self::Array { item_format: self_item, length: Some(self_length) }, Self::Array { item_format: other_item, length: Some(other_length) }) => {
                self_length == other_length && self_item.can_coerce_to(other_item.as_ref(), is_concrete)
            },
            _ => false
        }
    }

    pub fn requires_bitcast_to(&self, other: &Self) -> bool {
        let self_unqualified = self.as_unqualified();
        let other_unqualified = other.as_unqualified();
        self_unqualified != other_unqualified && match (self_unqualified, other_unqualified) {
            (Self::Pointer(self_pointee), Self::Pointer(other_pointee)) => {
                self_pointee.requires_bitcast_to(other_pointee)
            },
            (Self::Array { length: _, .. }, Self::Array { length: None, .. }) => {
                true
            },
            (Self::Array { item_format: self_item, length: Some(self_length) }, Self::Array { item_format: other_item, length: Some(other_length) }) => {
                self_length != other_length || self_item.requires_bitcast_to(other_item.as_ref())
            },
            _ => true
        }
    }

    pub fn expect_constant_sized(&self) -> crate::Result<()> {
        match self.size() {
            Some(_) => {
                Ok(())
            },
            None => {
                Err(crate::RawError::new(format!("cannot use type '{}' here, as it does not have a known size at compile time (did you mean to use a pointer?)", self.rich_name())).into_boxed())
            },
        }
    }

    pub fn rich_name(&self) -> String {
        match self {
            Self::Never => {
                String::from("never")
            }
            Self::Void => {
                String::from("void")
            },
            Self::Mutable(format) => {
                format!("mut {format}", format = format.rich_name())
            },
            Self::Boolean => {
                String::from("bool")
            },
            Self::Integer { size, signed: true } => {
                format!("i{bits}", bits = size * 8)
            },
            Self::Integer { size, signed: false } => {
                format!("u{bits}", bits = size * 8)
            },
            Self::Pointer(pointee_format) => {
                format!("*{pointee_format}", pointee_format = pointee_format.rich_name())
            },
            Self::Array { item_format, length: Some(length) } => {
                format!("[{item_format}; {length}]", item_format = item_format.rich_name())
            },
            Self::Array { item_format, length: None } => {
                format!("[{item_format}]", item_format = item_format.rich_name())
            },
            Self::Function { return_format, parameters, is_varargs, .. } => {
                let mut name = String::from("function(");
                let mut parameters_iter = parameters.iter();
                if let Some(parameter) = parameters_iter.next() {
                    name = format!("{name}{parameter}", parameter = parameter.rich_name());
                    for parameter in parameters_iter {
                        name = format!("{name}, {parameter}", parameter = parameter.rich_name());
                    }
                    if *is_varargs {
                        name = format!("{name}, ..");
                    }
                }
                else if *is_varargs {
                    name = format!("{name}..");
                }
                format!("{name}) -> {return_format}", return_format = return_format.rich_name())
            },
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never | Self::Void => {
                write!(f, "void")
            },
            Self::Boolean => {
                write!(f, "i1")
            },
            Self::Mutable(format) => {
                write!(f, "{format}")
            },
            Self::Integer { size, .. } => {
                write!(f, "i{bits}", bits = size * 8)
            },
            Self::Pointer(pointee_format) => {
                if let Self::Void = pointee_format.as_unqualified() {
                    write!(f, "i8*") // I wanted to use `ptr`, but LLVM complains unless -opaque-pointers is enabled
                }
                else {
                    write!(f, "{pointee_format}*")
                }
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
                if let Some(parameter) = parameters_iter.next() {
                    write!(f, "{parameter}")?;
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
pub enum IntegerValue {
    Signed8(i8),
    Unsigned8(u8),
    Signed16(i16),
    Unsigned16(u16),
    Signed32(i32),
    Unsigned32(u32),
    Signed64(i64),
    Unsigned64(u64),
}

impl IntegerValue {
    pub fn new(raw: u64, format: &Format) -> Option<Self> {
        match format.as_unqualified() {
            Format::Integer { size: 1, signed: true } => Some(Self::Signed8(raw as i8)),
            Format::Integer { size: 1, signed: false } => Some(Self::Unsigned8(raw as u8)),
            Format::Integer { size: 2, signed: true } => Some(Self::Signed16(raw as i16)),
            Format::Integer { size: 2, signed: false } => Some(Self::Unsigned16(raw as u16)),
            Format::Integer { size: 4, signed: true } => Some(Self::Signed32(raw as i32)),
            Format::Integer { size: 4, signed: false } => Some(Self::Unsigned32(raw as u32)),
            Format::Integer { size: 8, signed: true } => Some(Self::Signed64(raw as i64)),
            Format::Integer { size: 8, signed: false } => Some(Self::Unsigned64(raw as u64)),
            _ => None
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Signed8(_) | Self::Unsigned8(_) => 1,
            Self::Signed16(_) | Self::Unsigned16(_) => 2,
            Self::Signed32(_) | Self::Unsigned32(_) => 4,
            Self::Signed64(_) | Self::Unsigned64(_) => 8,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Self::Signed8(_) | Self::Signed16(_) | Self::Signed32(_) | Self::Signed64(_) => true,
            Self::Unsigned8(_) | Self::Unsigned16(_) | Self::Unsigned32(_) | Self::Unsigned64(_) => false,
        }
    }

    pub fn format(&self) -> Format {
        Format::Integer {
            size: self.size(),
            signed: self.is_signed(),
        }
    }
}

impl fmt::Display for IntegerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signed8(value) => write!(f, "{value}"),
            Self::Unsigned8(value) => write!(f, "{value}"),
            Self::Signed16(value) => write!(f, "{value}"),
            Self::Unsigned16(value) => write!(f, "{value}"),
            Self::Signed32(value) => write!(f, "{value}"),
            Self::Unsigned32(value) => write!(f, "{value}"),
            Self::Signed64(value) => write!(f, "{value}"),
            Self::Unsigned64(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Register {
    name: String,
    format: Format,
    is_global: bool,
}

impl Register {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn format(&self) -> &Format {
        &self.format
    }

    pub fn format_mut(&mut self) -> &mut Format {
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
        }
        else {
            write!(f, "%{}", self.name)
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Constant {
    ZeroInitializer(Format),
    NullPointer(Format),
    Boolean(bool),
    Integer(IntegerValue),
    String(token::StringValue),
    Array(Vec<Constant>, Format),
    Register(Register),
    Indirect {
        pointer: Box<Constant>,
        loaded_format: Format,
    },
    BitwiseCast {
        value: Box<Constant>,
        to_format: Format,
    },
    GetElementPointer {
        element_format: Format,
        aggregate_format: Format,
        pointer: Box<Constant>,
        indices: Vec<Constant>,
    },
}

impl Constant {
    pub fn format(&self) -> Format {
        match self {
            Self::ZeroInitializer(format) => format.clone(),
            Self::NullPointer(format) => format.clone(),
            Self::Boolean(_) => Format::Boolean,
            Self::Integer(value) => value.format(),
            Self::String(value) => Format::Array {
                item_format: Box::new(Format::Integer { size: 1, signed: false }),
                length: Some(value.len()),
            },
            Self::Array(value, format) => Format::Array {
                item_format: Box::new(format.clone()),
                length: Some(value.len()),
            },
            Self::Register(register) => register.format().clone(),
            Self::Indirect { loaded_format, .. } => loaded_format.clone(),
            Self::BitwiseCast { to_format, .. } => to_format.clone(),
            Self::GetElementPointer { element_format, .. } => element_format.clone().into_pointer(),
        }
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroInitializer(_) => write!(f, "zeroinitializer"),
            Self::NullPointer(_) => write!(f, "null"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Integer(value) => write!(f, "{value}"),
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
            Self::Register(register) => write!(f, "{register}"),
            Self::Indirect { .. } => write!(f, "<ERROR indirect constant>"),
            Self::BitwiseCast { value, to_format } => {
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
        loaded_format: Format,
    },
}

impl Value {
    pub fn format(&self) -> Format {
        match self {
            Self::Never => Format::Never,
            Self::Void => Format::Void,
            Self::Constant(constant) => constant.format(),
            Self::Register(register) => register.format().clone(),
            Self::Indirect { loaded_format, .. } => loaded_format.clone(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never => write!(f, "<ERROR never value>"),
            Self::Void => write!(f, "<ERROR void value>"),
            Self::Constant(constant) => write!(f, "{constant}"),
            Self::Register(register) => write!(f, "{register}"),
            Self::Indirect { .. } => write!(f, "<ERROR indirect value>"),
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
pub struct FunctionContext {
    name: String,
    return_format: Format,
}

impl FunctionContext {
    pub fn new(name: String, return_format: Format) -> Self {
        Self {
            name,
            return_format,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn return_format(&self) -> &Format {
        &self.return_format
    }
}

#[derive(Clone, Debug)]
pub struct ScopeContext {
    function_context: Option<FunctionContext>,
    break_label: Option<Label>,
    continue_label: Option<Label>,
}

impl ScopeContext {
    pub fn new() -> Self {
        Self {
            function_context: None,
            break_label: None,
            continue_label: None,
        }
    }

    pub fn is_global(&self) -> bool {
        self.function_context.is_none()
    }

    pub fn function(&self) -> Option<&FunctionContext> {
        self.function_context.as_ref()
    }

    pub fn break_label(&self) -> Option<&Label> {
        self.break_label.as_ref()
    }

    pub fn continue_label(&self) -> Option<&Label> {
        self.continue_label.as_ref()
    }

    pub fn enter_function(&self, name: String, return_format: Format) -> Self {
        let mut new_scope = self.clone();
        new_scope.function_context = Some(FunctionContext::new(name, return_format));
        new_scope
    }

    pub fn enter_loop(&self, break_label: Label, continue_label: Label) -> Self {
        let mut new_scope = self.clone();
        new_scope.break_label = Some(break_label);
        new_scope.continue_label = Some(continue_label);
        new_scope
    }
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
    const DEFAULT_SYMBOL_TABLE_CAPACITY: usize = 64;

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

    pub fn new_anonymous_register(&mut self, format: Format) -> Register {
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
            name: format!(".block.{id}"),
        }
    }

    pub fn new_anonymous_constant(&mut self, format: Format) -> Register {
        let id = self.next_anonymous_constant_id;
        self.next_anonymous_constant_id += 1;

        Register {
            name: format!(".const.{id}"),
            format: format.into_pointer(),
            is_global: true,
        }
    }

    pub fn get_symbol_table(&self, scope: &ScopeContext) -> &info::SymbolTable {
        if scope.is_global() {
            &self.global_symbol_table
        }
        else {
            &self.local_symbol_table
        }
    }

    pub fn define_symbol(&mut self, scope: &ScopeContext, symbol: info::Symbol) {
        if scope.is_global() {
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
    
    pub fn get_format_from_node(&self, type_node: &ast::TypeNode, allow_mutable: bool, allow_unsized: bool) -> crate::Result<Format> {
        let format = match type_node {
            ast::TypeNode::Named(name) => {
                match name.as_str() {
                    "never" => Format::Never,
                    "void" => Format::Void,
                    "bool" => Format::Boolean,
                    "i8" => Format::Integer { size: 1, signed: true },
                    "u8" => Format::Integer { size: 1, signed: false },
                    "i16" => Format::Integer { size: 2, signed: true },
                    "u16" => Format::Integer { size: 2, signed: false },
                    "i32" => Format::Integer { size: 4, signed: true },
                    "u32" => Format::Integer { size: 4, signed: false },
                    "i64" => Format::Integer { size: 8, signed: true },
                    "u64" => Format::Integer { size: 8, signed: false },
                    _ => return Err(crate::RawError::new(format!("unrecognized type name '{name}'")).into_boxed())
                }
            },
            ast::TypeNode::Mutable(mutable_type) => {
                if allow_mutable {
                    self.get_format_from_node(mutable_type.as_ref(), false, allow_unsized)?
                        .into_mutable()
                }
                else {
                    return Err(crate::RawError::new(String::from("mutable types are not allowed here")).into_boxed());
                }
            },
            ast::TypeNode::Pointer(pointee_type) => {
                self.get_format_from_node(pointee_type.as_ref(), true, true)?
                    .into_pointer()
            },
            ast::TypeNode::Array(item_type, Some(length)) => {
                if let ast::Node::Literal(token::Literal::Integer(length)) = length.as_ref() {
                    Format::Array {
                        item_format: Box::new(self.get_format_from_node(item_type.as_ref(), false, false)?),
                        length: Some(*length as usize),
                    }
                }
                else {
                    return Err(crate::RawError::new(String::from("array length must be constant")).into_boxed());
                }
            },
            ast::TypeNode::Array(item_type, None) => {
                Format::Array {
                    item_format: Box::new(self.get_format_from_node(item_type.as_ref(), false, false)?),
                    length: None,
                }
            },
        };

        if !allow_unsized {
            format.expect_constant_sized()?;
        }

        Ok(format)
    }

    pub fn enforce_format(&mut self, value: Value, target_format: &Format) -> crate::Result<Value> {
        let got_format = value.format();

        if &got_format == target_format {
            Ok(value)
        }
        else if got_format.can_coerce_to(target_format, true) {
            if !got_format.requires_bitcast_to(target_format) {
                Ok(value)
            }
            else if let Value::Constant(constant) = value {
                Ok(Value::Constant(Constant::BitwiseCast {
                    value: Box::new(constant),
                    to_format: target_format.clone(),
                }))
            }
            else {
                let result = self.new_anonymous_register(target_format.clone());
                self.emitter.emit_bitwise_cast(&result, &value)?;

                Ok(Value::Register(result))
            }
        }
        else {
            Err(crate::RawError::new(format!(
                "expected a value of type '{target_format}', got '{got_format}' instead",
                target_format = target_format.rich_name(),
                got_format = got_format.rich_name(),
            )).into_boxed())
        }
    }

    pub fn enforce_constant_format(&self, constant: Constant, target_format: &Format) -> crate::Result<Constant> {
        let got_format = constant.format();

        if &got_format == target_format {
            Ok(constant)
        }
        else if got_format.can_coerce_to(target_format, true) {
            if got_format.requires_bitcast_to(target_format) {
                Ok(Constant::BitwiseCast {
                    value: Box::new(constant),
                    to_format: target_format.clone(),
                })
            }
            else {
                Ok(constant)
            }
        }
        else {
            Err(crate::RawError::new(format!(
                "expected a constant of type '{target_format}', got '{got_format}' instead",
                target_format = target_format.rich_name(),
                got_format = got_format.rich_name(),
            )).into_boxed())
        }
    }

    pub fn change_format(&mut self, value: Value, target_format: &Format) -> crate::Result<Value> {
        let original_format = value.format();

        if target_format == &original_format {
            Ok(value)
        }
        else {
            match (original_format.as_unqualified(), target_format.as_unqualified()) {
                (Format::Pointer(_), Format::Pointer(_)) => {
                    let result = self.new_anonymous_register(target_format.clone());
                    self.emitter.emit_bitwise_cast(&result, &value)?;

                    Ok(Value::Register(result))
                },
                (Format::Integer { size: from_size, .. }, Format::Integer { size: to_size, .. }) => {
                    if to_size > from_size {
                        let result = self.new_anonymous_register(target_format.clone());
                        self.emitter.emit_extension(&result, &value)?;

                        Ok(Value::Register(result))
                    }
                    else if to_size < from_size {
                        let result = self.new_anonymous_register(target_format.clone());
                        self.emitter.emit_truncation(&result, &value)?;

                        Ok(Value::Register(result))
                    }
                    else {
                        Ok(value)
                    }
                },
                (Format::Integer { .. }, Format::Boolean) => {
                    let result = self.new_anonymous_register(Format::Boolean);
                    let zero = IntegerValue::new(0, &original_format).unwrap();
                    self.emitter.emit_cmp_not_equal(&result, &value, &Value::Constant(Constant::Integer(zero)))?;
                    
                    Ok(Value::Register(result))
                },
                (Format::Boolean, Format::Integer { .. }) => {
                    let result = self.new_anonymous_register(target_format.clone());
                    self.emitter.emit_zero_extension(&result, &value)?;
                    
                    Ok(Value::Register(result))
                },
                _ => {
                    Err(crate::RawError::new(format!(
                        "cannot convert from '{original_format}' to '{target_format}'",
                        original_format = original_format.rich_name(),
                        target_format = target_format.rich_name(),
                    )).into_boxed())
                }
            }
        }
    }

    pub fn coerce_to_rvalue(&mut self, value: Value) -> crate::Result<Value> {
        if let Value::Indirect { pointer, loaded_format } = value {
            let result = self.new_anonymous_register(loaded_format);
            self.emitter.emit_load(&result, pointer.as_ref())?;
            
            Ok(Value::Register(result))
        }
        else if let Value::Constant(Constant::Indirect { pointer, loaded_format }) = value {
            let result = self.new_anonymous_register(loaded_format);
            self.emitter.emit_load(&result, &Value::Constant(*pointer))?;

            Ok(Value::Register(result))
        }
        else {
            Ok(value)
        }
    }

    pub fn generate_node(&mut self, node: &ast::Node, context: &ScopeContext, expected_format: Option<Format>) -> crate::Result<Value> {
        if let Ok(constant) = self.generate_constant_node(node, context, expected_format.clone()) {
            return Ok(Value::Constant(constant));
        }

        let result = match node {
            ast::Node::Literal(literal) => match literal {
                token::Literal::Identifier(name) => {
                    self.get_symbol(name)?.value().clone()
                },
                token::Literal::Integer(value) => {
                    let format = expected_format.clone().unwrap_or(Format::Integer { size: 4, signed: true });
                    let value = IntegerValue::new(*value, &format)
                        .ok_or_else(|| crate::RawError::new(format!("'{value}' cannot be used as a value of type '{}'", format.rich_name())).into_boxed())?;
                    Value::Constant(Constant::Integer(value))
                },
                token::Literal::Boolean(value) => {
                    Value::Constant(Constant::Boolean(*value))
                },
                token::Literal::NullPointer => {
                    Value::Constant(Constant::NullPointer(expected_format.clone().unwrap_or_else(Format::opaque_pointer)))
                },
                token::Literal::String(value) => {
                    let constant = Constant::String(value.clone());
                    let pointer = self.new_anonymous_constant(constant.format());

                    self.emitter.emit_anonymous_constant(&pointer, &constant)?;

                    Value::Constant(Constant::Register(pointer))
                },
            },
            ast::Node::Unary { operation, operand } => match operation {
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
                    let operand = self.generate_node(operand.as_ref(), context, Some(Format::Boolean))?;
                    let operand = self.coerce_to_rvalue(operand)?;
                    let result = self.new_anonymous_register(Format::Boolean);

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
                    let operand = self.generate_node(operand.as_ref(), context, expected_format.clone().map(Format::into_pointer))?;
                    let operand = self.coerce_to_rvalue(operand)?;
                    
                    if let Format::Pointer(pointee_format) = operand.format().as_unqualified() {
                        if let Value::Constant(constant) = operand {
                            Value::Constant(Constant::Indirect {
                                pointer: Box::new(constant),
                                loaded_format: pointee_format.as_ref().clone(),
                            })
                        }
                        else {
                            Value::Indirect {
                                pointer: Box::new(operand),
                                loaded_format: pointee_format.as_ref().clone(),
                            }
                        }
                    }
                    else {
                        return Err(crate::RawError::new(format!("cannot dereference value of type '{}'", operand.format().rich_name())).into_boxed());
                    }
                },
                ast::UnaryOperation::GetSize => {
                    if let ast::Node::Type(type_node) = operand.as_ref() {
                        let format = self.get_format_from_node(type_node, true, false)?;

                        if let Some(size) = format.size() {
                            Value::Constant(Constant::Integer(IntegerValue::Unsigned64(size as u64)))
                        }
                        else {
                            return Err(crate::RawError::new(format!("type '{type_node}' does not have a size")).into_boxed());
                        }
                    }
                    else {
                        return Err(crate::RawError::new(String::from("invalid operand for 'sizeof'")).into_boxed());
                    }
                },
                ast::UnaryOperation::GetAlign => todo!(),
            },
            ast::Node::Binary { operation, lhs, rhs } => match operation {
                ast::BinaryOperation::Subscript => {
                    let lhs = self.generate_node(lhs.as_ref(), context, None)?;
                    let rhs = self.generate_node(rhs.as_ref(), context, None)?;
                    let rhs = self.coerce_to_rvalue(rhs)?;
                    
                    if !(matches!(rhs.format().as_unqualified(), Format::Integer { .. })) {
                        return Err(crate::RawError::new(String::from("expected an integer index")).into_boxed());
                    }

                    // Brain hurty so I made big list:

                    // mut [T; N] <=> [N x T] : not allowed (for now) : error
                    // mut &[T; N] <=> [N x T]* : getelementptr instruction (0, index) : indirect value
                    // mut *[T; N] <=> [N x T]* : getelementptr instruction (0, index) : indirect value
                    // mut &*[T; N] <=> [N x T]** : load -> getelementptr instruction (0, index) : indirect value
                    // mut [T] <=> T : not possible : error
                    // mut &[T] <=> T* : getelementptr instruction (index) : indirect value
                    // mut *[T] <=> T* : getelementptr instruction (index) : indirect value
                    // mut &*[T] <=> T** : load -> getelementptr instruction (index) : indirect value
                    // const [T; N] <=> [N x T] : not allowed (for now) : error
                    // const &[T; N] <=> [N x T]* : getelementptr expression (0, index) : indirect constant
                    // const *[T; N] <=> [N x T]* : getelementptr expression (0, index) : indirect constant
                    // const &*[T; N] <=> [N x T]** : (only in non-const context) load -> getelementptr instruction (0, index) : indirect value
                    // const [T] <=> T : not possible : error
                    // const &[T] <=> T* : getelementptr expression (index) : indirect constant
                    // const *[T] <=> T* : getelementptr expression (index) : indirect constant
                    // const &*[T] <=> T** : (only in non-const context) load -> getelementptr instruction (index) : indirect value

                    // Inputs: lvalue/rvalue, is behind pointer, sized/unsized, lhs constant, rhs constant
                    // Outputs: whether to load, expression vs. instruction, (0, index) vs. (index)

                    // TL;DR: I think I've way overcomplicated this but I don't know what else to do

                    let lhs_format = lhs.format();
                    let cannot_index_error = || crate::RawError::new(format!("cannot index value of type '{}'", lhs_format.rich_name())).into_boxed();

                    // This is incredibly nasty and repetitive... could signify a need for a rethink of Value/Constant and/or Format
                    match (lhs, rhs) {
                        (Value::Constant(lhs), Value::Constant(rhs)) => match lhs {
                            Constant::Indirect { pointer, loaded_format } => match loaded_format.as_unqualified() {
                                Format::Array { item_format, length } => {
                                    // const &[T; N], const &[T]
                                    let indices = match length {
                                        Some(_) => vec![Constant::Integer(IntegerValue::Signed32(0)), rhs],
                                        None => vec![rhs],
                                    };

                                    let element_pointer = Constant::GetElementPointer {
                                        element_format: item_format.as_ref().clone().into_pointer(),
                                        aggregate_format: loaded_format.clone(),
                                        pointer,
                                        indices,
                                    };

                                    Value::Constant(Constant::Indirect {
                                        pointer: Box::new(element_pointer),
                                        loaded_format: item_format.as_ref().clone(),
                                    })
                                },
                                Format::Pointer(pointee_format) => match pointee_format.as_unqualified() {
                                    Format::Array { item_format, length } => {
                                        // const &*[T; N], const &*[T]
                                        let element_format = match pointee_format.as_ref() {
                                            Format::Mutable(_) => item_format.as_ref().clone().into_mutable(),
                                            _ => item_format.as_ref().clone()
                                        };
                                        let loaded_pointer = self.new_anonymous_register(loaded_format.clone());
                                        let element_pointer = self.new_anonymous_register(element_format.clone().into_pointer());
                                        let indices = match length {
                                            Some(_) => vec![Value::Constant(Constant::Integer(IntegerValue::Signed32(0))), Value::Constant(rhs)],
                                            None => vec![Value::Constant(rhs)],
                                        };

                                        self.emitter.emit_load(&loaded_pointer, &Value::Constant(*pointer))?;
                                        self.emitter.emit_get_element_pointer(
                                            &element_pointer,
                                            pointee_format.as_ref(),
                                            &Value::Register(loaded_pointer),
                                            &indices,
                                        )?;

                                        Value::Indirect {
                                            pointer: Box::new(Value::Register(element_pointer)),
                                            loaded_format: element_format,
                                        }
                                    },
                                    _ => return Err(cannot_index_error())
                                },
                                _ => return Err(cannot_index_error())
                            },
                            Constant::Register(register) => match register.format().clone().as_unqualified() {
                                Format::Pointer(pointee_format) => match pointee_format.as_unqualified() {
                                    Format::Array { item_format, length } => {
                                        // const *[T; N], const *[T]
                                        let indices = match length {
                                            Some(_) => vec![Constant::Integer(IntegerValue::Signed32(0)), rhs],
                                            None => vec![rhs],
                                        };

                                        let element_pointer = Constant::GetElementPointer {
                                            element_format: item_format.as_ref().clone().into_pointer(),
                                            aggregate_format: pointee_format.as_ref().clone(),
                                            pointer: Box::new(Constant::Register(register)),
                                            indices,
                                        };

                                        Value::Constant(Constant::Indirect {
                                            pointer: Box::new(element_pointer),
                                            loaded_format: item_format.as_ref().clone(),
                                        })
                                    },
                                    _ => return Err(cannot_index_error())
                                },
                                _ => return Err(cannot_index_error())
                            },
                            _ => return Err(cannot_index_error())
                        },
                        (lhs, rhs) => match lhs {
                            Value::Indirect { pointer, loaded_format } => match loaded_format.as_unqualified() {
                                Format::Array { item_format, length } => {
                                    // &[T; N], &[T]
                                    let element_pointer = self.new_anonymous_register(item_format.as_ref().clone().into_pointer());
                                    let indices = match length {
                                        Some(_) => vec![Value::Constant(Constant::Integer(IntegerValue::Signed32(0))), rhs],
                                        None => vec![rhs],
                                    };

                                    self.emitter.emit_get_element_pointer(
                                        &element_pointer,
                                        &loaded_format,
                                        pointer.as_ref(),
                                        &indices,
                                    )?;

                                    Value::Indirect {
                                        pointer: Box::new(Value::Register(element_pointer)),
                                        loaded_format: item_format.as_ref().clone(),
                                    }
                                },
                                Format::Pointer(pointee_format) => match pointee_format.as_unqualified() {
                                    Format::Array { item_format, length } => {
                                        // &*[T; N], &*[T]
                                        let element_format = match pointee_format.as_ref() {
                                            Format::Mutable(_) => item_format.as_ref().clone().into_mutable(),
                                            _ => item_format.as_ref().clone()
                                        };
                                        let loaded_pointer = self.new_anonymous_register(loaded_format.clone());
                                        let element_pointer = self.new_anonymous_register(element_format.clone().into_pointer());
                                        let indices = match length {
                                            Some(_) => vec![Value::Constant(Constant::Integer(IntegerValue::Signed32(0))), rhs],
                                            None => vec![rhs],
                                        };

                                        self.emitter.emit_load(&loaded_pointer, pointer.as_ref())?;
                                        self.emitter.emit_get_element_pointer(
                                            &element_pointer,
                                            pointee_format.as_ref(),
                                            &Value::Register(loaded_pointer),
                                            &indices,
                                        )?;

                                        Value::Indirect {
                                            pointer: Box::new(Value::Register(element_pointer)),
                                            loaded_format: element_format,
                                        }
                                    },
                                    _ => return Err(cannot_index_error())
                                },
                                _ => return Err(cannot_index_error())
                            },
                            Value::Register(register) => match register.format().clone().as_unqualified() {
                                Format::Pointer(pointee_format) => match pointee_format.as_unqualified() {
                                    Format::Array { item_format, length } => {
                                        // *[T; N], *[T]
                                        let element_pointer = self.new_anonymous_register(item_format.as_ref().clone().into_pointer());
                                        let indices = match length {
                                            Some(_) => vec![Value::Constant(Constant::Integer(IntegerValue::Signed32(0))), rhs],
                                            None => vec![rhs],
                                        };

                                        self.emitter.emit_get_element_pointer(
                                            &element_pointer,
                                            pointee_format.as_ref(),
                                            &Value::Register(register),
                                            &indices,
                                        )?;

                                        Value::Indirect {
                                            pointer: Box::new(Value::Register(element_pointer)),
                                            loaded_format: item_format.as_ref().clone(),
                                        }
                                    },
                                    _ => return Err(cannot_index_error())
                                },
                                _ => return Err(cannot_index_error())
                            },
                            _ => return Err(cannot_index_error())
                        }
                    }
                },
                ast::BinaryOperation::Access => todo!(),
                ast::BinaryOperation::DerefAccess => todo!(),
                ast::BinaryOperation::Convert => {
                    if let ast::Node::Type(type_node) = rhs.as_ref() {
                        let value = self.generate_node(lhs.as_ref(), context, None)?;
                        let value = self.coerce_to_rvalue(value)?;
                        
                        let target_format = self.get_format_from_node(type_node, false, false)?;
    
                        self.change_format(value, &target_format)?
                    }
                    else {
                        return Err(crate::RawError::new(String::from("expected a type following 'as'")).into_boxed());
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
            },
            ast::Node::Call { callee, arguments } => {
                let callee = self.generate_node(callee.as_ref(), context, None)?;
                let callee = self.coerce_to_rvalue(callee)?;

                if let Format::Function { return_format, parameters, is_varargs, .. } = callee.format() {
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
                        Format::Never => {
                            self.emitter.emit_function_call(None, &callee, &argument_values)?;
                            self.emitter.emit_unreachable()?;

                            Value::Never
                        },
                        Format::Void => {
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
                let condition = self.generate_node(condition.as_ref(), context, Some(Format::Boolean))?;
                let condition = self.coerce_to_rvalue(condition)?;

                let consequent_label = self.new_anonymous_label();
                let alternative_label = self.new_anonymous_label();

                self.emitter.emit_conditional_branch(&condition, &consequent_label, &alternative_label)?;
                
                if let Some(alternative) = alternative {
                    let mut tail_label = None;

                    self.emitter.emit_label(&consequent_label)?;
                    let consequent_value = self.generate_node(consequent.as_ref(), context, None)?;
                    if &consequent_value != &Value::Never {
                        tail_label = tail_label.or_else(|| Some(self.new_anonymous_label()));
                        self.emitter.emit_unconditional_branch(tail_label.as_ref().unwrap())?;
                    }

                    self.emitter.emit_label(&alternative_label)?;
                    let alternative_value = self.generate_node(alternative.as_ref(), context, None)?;
                    if &alternative_value != &Value::Never {
                        tail_label = tail_label.or_else(|| Some(self.new_anonymous_label()));
                        self.emitter.emit_unconditional_branch(tail_label.as_ref().unwrap())?;
                    }

                    if let Some(tail_label) = tail_label {
                        self.emitter.emit_label(&tail_label)?;

                        Value::Void
                    }
                    else {
                        Value::Never
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

                let condition = self.generate_node(condition.as_ref(), context, Some(Format::Boolean))?;
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
                let return_format = context.function().map(|function| function.return_format())
                    .ok_or_else(|| crate::RawError::new(String::from("'return' outside of function")).into_boxed())?;

                if let Some(value) = value {
                    if let Format::Void = return_format {
                        return Err(crate::RawError::new(String::from("returning without a value from a non-void function")).into_boxed());
                    }
                    else {
                        let value = self.generate_node(value.as_ref(), context, Some(return_format.clone()))?;
                        let value = self.coerce_to_rvalue(value)?;

                        self.emitter.emit_return(Some(&value))?;
                    }
                }
                else {
                    if let Format::Void = return_format {
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
                let format = self.get_format_from_node(value_type, true, false)?;
                let (symbol, pointer) = self.get_symbol_table(context).create_indirect_symbol(name.clone(), format.clone());

                if context.is_global() {
                    let init_value = if let Some(node) = value {
                        self.generate_constant_node(node.as_ref(), context, Some(format.clone()))?
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
            ast::Node::Constant { name, value_type, value } => {
                let format = self.get_format_from_node(value_type, false, false)?;

                if let Some(function) = context.function() {
                    let constant = self.generate_constant_node(value.as_ref(), context, Some(format.clone()))?;
                    let (symbol, pointer) = self.get_symbol_table(context).create_indirect_local_constant_symbol(name.clone(), format.clone(), function.name());

                    self.emitter.emit_global_allocation(&pointer, &constant, true)?;
                    
                    self.define_symbol(context, symbol);
                }
                else {
                    let constant = self.generate_constant_node(value.as_ref(), context, Some(format.clone()))?;
                    let (symbol, pointer) = self.global_symbol_table.create_indirect_symbol(name.clone(), format.clone());

                    self.emitter.emit_global_allocation(&pointer, &constant, true)?;

                    self.define_symbol(context, symbol);
                }

                Value::Void
            },
            ast::Node::Function { name, parameters, is_varargs, return_type, body: None } => {
                // TODO: maybe do multiple passes of the source file to avoid the need for forward declarations
                let return_format = self.get_format_from_node(return_type, false, false)?;
                let parameter_formats: Vec<Format> = Result::from_iter(parameters.iter().map(|(_, parameter_type)| {
                    self.get_format_from_node(parameter_type, true, false)
                }))?;

                let format = Format::Function {
                    is_defined: false,
                    return_format: Box::new(return_format.clone()),
                    parameters: parameter_formats.clone(), 
                    is_varargs: *is_varargs,
                };

                let function_register;
                if let Some(previous_symbol) = self.global_symbol_table.find(name) {
                    if let Format::Function {
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
                let return_format = self.get_format_from_node(return_type, false, false)?;
                let parameter_formats: Vec<Format> = Result::from_iter(parameters.iter().map(|(_, parameter_type)| {
                    self.get_format_from_node(parameter_type, true, false)
                }))?;
                
                let parameter_names_iter = parameters.iter().map(
                    |(name, _)| name.clone()
                );

                self.local_symbol_table.clear();
                self.next_anonymous_register_id = 0;
                self.next_anonymous_label_id = 0;
                let function_context = context.clone().enter_function(name.clone(), return_format.clone());

                let parameter_handles: Vec<_> = std::iter::zip(parameter_names_iter, parameter_formats.iter())
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

                let format = Format::Function {
                    is_defined: true,
                    return_format: Box::new(return_format.clone()),
                    parameters: parameter_formats.clone(), 
                    is_varargs: *is_varargs,
                };

                let function_register;
                if let Some(previous_symbol) = self.global_symbol_table.find_mut(name) {
                    if let Format::Function {
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
                let entry_label = self.new_anonymous_label();
                self.emitter.emit_label(&entry_label)?;

                for (input_register, symbol, pointer) in parameter_handles {
                    self.emitter.emit_local_allocation(&pointer, input_register.format())?;
                    self.emitter.emit_store(&Value::Register(input_register), &Value::Register(pointer))?;
                    self.define_symbol(&function_context, symbol);
                }

                let body_result = self.generate_node(body.as_ref(), &function_context, None)?;

                if &body_result != &Value::Never {
                    if let Format::Void = &return_format {
                        self.emitter.emit_return(None)?;
                    }
                    else {
                        return Err(crate::RawError::new(format!("non-void function '{name}' could finish without returning a value")).into_boxed());
                    }
                }

                self.emitter.emit_function_exit()?;

                Value::Void
            },
            _ => {
                return Err(crate::RawError::new(String::from("unexpected expression")).into_boxed())
            }
        };

        if let Some(expected_format) = &expected_format {
            self.enforce_format(result, expected_format)
        }
        else {
            Ok(result)
        }
    }

    fn generate_binary_arithmetic_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<Format>) -> crate::Result<(Register, Value, Value)> {
        let lhs = self.generate_node(lhs, context, expected_format.clone())?;
        let lhs = self.coerce_to_rvalue(lhs)?;

        let rhs = self.generate_node(rhs, context, Some(lhs.format()))?;
        let rhs = self.coerce_to_rvalue(rhs)?;

        let result = self.new_anonymous_register(expected_format.unwrap_or_else(|| lhs.format()));

        Ok((result, lhs, rhs))
    }

    fn generate_comparison_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext) -> crate::Result<(Register, Value, Value)> {
        let lhs = self.generate_node(lhs, context, None)?;
        let lhs = self.coerce_to_rvalue(lhs)?;

        let rhs = self.generate_node(rhs, context, Some(lhs.format()))?;
        let rhs = self.coerce_to_rvalue(rhs)?;

        let result = self.new_anonymous_register(Format::Boolean);

        Ok((result, lhs, rhs))
    }

    fn generate_assignment(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<Format>) -> crate::Result<Value> {
        let lhs = self.generate_node(lhs, context, expected_format.clone())?;

        match lhs {
            Value::Indirect { pointer, loaded_format } => {
                if let Format::Mutable(_) = &loaded_format {
                    let rhs = self.generate_node(rhs, context, Some(loaded_format))?;
                    let rhs = self.coerce_to_rvalue(rhs)?;

                    self.emitter.emit_store(&rhs, pointer.as_ref())?;

                    Ok(rhs)
                }
                else {
                    Err(crate::RawError::new(format!("cannot mutate value of type '{}' as it is not 'mut'", loaded_format.rich_name())).into_boxed())
                }
            },
            _ => {
                Err(crate::RawError::new(String::from("expected an lvalue")).into_boxed())
            }
        }
    }

    pub fn generate_constant_node(&mut self, node: &ast::Node, _context: &ScopeContext, expected_format: Option<Format>) -> crate::Result<Constant> {
        let mut constant_id = self.next_anonymous_constant_id;
        let (constant, intermediate_constants) = self.fold_as_constant(node, &mut constant_id, expected_format)?;
        self.next_anonymous_constant_id = constant_id;

        for (pointer, intermediate_constant) in intermediate_constants {
            self.emitter.emit_anonymous_constant(&pointer, &intermediate_constant)?;
        }

        Ok(constant)
    }

    pub fn fold_as_constant(&self, node: &ast::Node, constant_id: &mut usize, expected_format: Option<Format>) -> crate::Result<(Constant, Vec<(Register, Constant)>)> {
        let mut new_intermediate_constant = |constant: Constant| {
            let pointer = Register {
                name: format!(".const.{}", constant_id),
                format: constant.format().into_pointer(),
                is_global: true,
            };
            *constant_id += 1;
            (pointer, constant)
        };

        let mut intermediate_constants = Vec::new();

        let constant = match node {
            ast::Node::Literal(literal) => {
                match literal {
                    token::Literal::Identifier(name) => {
                        if let Value::Constant(constant) = self.get_symbol(name)?.value() {
                            constant.clone()
                        }
                        else {
                            return Err(crate::RawError::new(format!("'{name}' is not constant and cannot be used in a constant expression")).into_boxed());
                        }
                    },
                    token::Literal::Integer(value) => {
                        let format = expected_format.clone().unwrap_or(Format::Integer { size: 4, signed: true });
                        let value = IntegerValue::new(*value, &format)
                            .ok_or_else(|| crate::RawError::new(format!("'{value}' cannot be used as a value of type '{}'", format.rich_name())).into_boxed())?;
                        Constant::Integer(value)
                    },
                    token::Literal::Boolean(value) => {
                        Constant::Boolean(*value)
                    },
                    token::Literal::NullPointer => {
                        Constant::NullPointer(expected_format.clone().unwrap_or_else(Format::opaque_pointer))
                    },
                    token::Literal::String(value) => {
                        let (pointer, constant) = new_intermediate_constant(Constant::String(value.clone()));
                        intermediate_constants.push((pointer.clone(), constant));

                        Constant::Register(pointer)
                    },
                }
            },
            _ => {
                return Err(crate::RawError::new(String::from("unexpected expression in constant")).into_boxed())
            }
        };

        if let Some(expected_format) = &expected_format {
            self.enforce_constant_format(constant, expected_format)
                .map(|constant| (constant, intermediate_constants))
        }
        else {
            Ok((constant, intermediate_constants))
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
