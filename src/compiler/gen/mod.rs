pub mod info;
pub mod llvm;

use crate::token;
use crate::ast;
use crate::Error;

use std::io::{Write, BufRead};
use std::fmt;

#[derive(Clone, PartialEq, Eq, Debug)]
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
    Structure {
        type_name: Option<String>,
        members: Vec<(String, Format)>,
    },
    Identified {
        identifier: String,
        type_name: String,
    },
    Function {
        is_defined: bool,
        return_format: Box<Format>,
        parameters: Vec<Format>,
        is_varargs: bool,
    },
    Type,
}

impl Format {
    pub fn opaque_pointer() -> Self {
        Self::Pointer(Box::new(Format::Void))
    }

    pub fn size(&self, symbol_table: &info::SymbolTable) -> Option<usize> {
        match self {
            Self::Never => Some(0),
            Self::Void => Some(0),
            Self::Mutable(format) => format.size(symbol_table),
            Self::Boolean => Some(1),
            Self::Integer { size, .. } => Some(*size),
            Self::Pointer(_) => Some(8),
            Self::Array { length, item_format } => {
                length.and_then(|length| item_format.size(symbol_table).map(|item_size| item_size * length))
            },
            Self::Structure { members, .. } => {
                let mut current_size = 0;
                let mut max_alignment = 0;

                for (_name, format) in members {
                    let alignment = format.alignment(symbol_table).unwrap();
                    max_alignment = max_alignment.max(alignment);

                    // Calculate padding
                    let intermediate_size = current_size + alignment - 1;
                    let padded_size = intermediate_size - intermediate_size % alignment;
                    current_size = padded_size + format.size(symbol_table).unwrap();
                }

                // Pad for the largest member alignment
                let intermediate_size = current_size + max_alignment - 1;
                let padded_size = intermediate_size - intermediate_size % max_alignment;
                Some(padded_size)
            },
            Self::Identified { type_name, .. } => {
                symbol_table.find(type_name).and_then(|symbol| {
                    if let Value::TypeDefinition(Some(format)) = symbol.value() {
                        format.size(symbol_table)
                    }
                    else {
                        None
                    }
                })
            },
            Self::Function { .. } => None,
            Self::Type => None,
        }
    }

    pub fn alignment(&self, symbol_table: &info::SymbolTable) -> Option<usize> {
        match self {
            Self::Never => None,
            Self::Void => None,
            Self::Mutable(format) => format.alignment(symbol_table),
            Self::Boolean => Some(1),
            Self::Integer { size, .. } => Some(*size),
            Self::Pointer(_) => Some(8),
            Self::Array { item_format, .. } => item_format.alignment(symbol_table),
            Self::Structure { members, .. } => {
                members.iter().map(|(_name, format)| format.alignment(symbol_table).unwrap()).max()
            },
            Self::Identified { type_name, .. } => {
                symbol_table.find(type_name).and_then(|symbol| {
                    if let Value::TypeDefinition(Some(format)) = symbol.value() {
                        format.size(symbol_table)
                    }
                    else {
                        None
                    }
                })
            },
            Self::Function { .. } => None,
            Self::Type => None,
        }
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self, Format::Mutable(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Format::Function { .. })
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Format::Type)
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

    pub fn expect_size(&self, symbol_table: &info::SymbolTable) -> crate::Result<usize> {
        self.size(symbol_table).ok_or_else(|| crate::RawError::new(format!(
            "cannot use type '{format}' here, as it does not have a known size at this time (did you mean to use a pointer?)",
            format = self.rich_name(),
        )).into_boxed())
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
            Self::Structure { type_name, members } => {
                if let Some(type_name) = type_name {
                    type_name.clone()
                }
                else {
                    let mut name = String::from("(");
                    let mut members_iter = members.iter();
                    if let Some((_, member_format)) = members_iter.next() {
                        name = format!("{name}{member_format}", member_format = member_format.rich_name());
                        for (_, member_format) in members_iter {
                            name = format!("{name}, {member_format}", member_format = member_format.rich_name());
                        }
                    }
                    format!("{name})")
                }
            },
            Self::Identified { type_name, .. } => {
                type_name.clone()
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
            Self::Type => {
                String::from("type")
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
            Self::Structure { members, .. } => {
                write!(f, "{{")?;
                let mut members_iter = members.iter();
                if let Some((_, member_format)) = members_iter.next() {
                    write!(f, " {member_format}")?;
                    for (_, member_format) in members_iter {
                        write!(f, ", {member_format}")?;
                    }
                    write!(f, " ")?;
                }
                write!(f, "}}")
            },
            Self::Identified { identifier, .. } => {
                write!(f, "%{identifier}")
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
            Self::Type => {
                write!(f, "<ERROR type format>")
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
    pub fn new(raw: i128, format: &Format) -> Option<Self> {
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

    pub fn expanded_value(&self) -> i128 {
        match self {
            &IntegerValue::Signed8(value) => value as i128,
            &IntegerValue::Unsigned8(value) => value as i128,
            &IntegerValue::Signed16(value) => value as i128,
            &IntegerValue::Unsigned16(value) => value as i128,
            &IntegerValue::Signed32(value) => value as i128,
            &IntegerValue::Unsigned32(value) => value as i128,
            &IntegerValue::Signed64(value) => value as i128,
            &IntegerValue::Unsigned64(value) => value as i128,
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name().partial_cmp(other.name())
    }
}

impl Ord for Register {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
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
    Undefined(Format),
    Poison(Format),
    ZeroInitializer(Format),
    NullPointer(Format),
    Boolean(bool),
    Integer(IntegerValue),
    String(token::StringValue),
    Array {
        items: Vec<Constant>,
        item_format: Format,
    },
    Structure {
        members: Vec<Constant>,
        format: Format,
    },
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
            Self::Undefined(format) => format.clone(),
            Self::Poison(format) => format.clone(),
            Self::ZeroInitializer(format) => format.clone(),
            Self::NullPointer(format) => format.clone(),
            Self::Boolean(_) => Format::Boolean,
            Self::Integer(value) => value.format(),
            Self::String(value) => Format::Array {
                item_format: Box::new(Format::Integer { size: 1, signed: false }),
                length: Some(value.len()),
            },
            Self::Array { items, item_format } => Format::Array {
                item_format: Box::new(item_format.clone()),
                length: Some(items.len()),
            },
            Self::Structure { format, .. } => format.clone(),
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
            Self::Undefined(_) => write!(f, "undef"),
            Self::Poison(_) => write!(f, "poison"),
            Self::ZeroInitializer(_) => write!(f, "zeroinitializer"),
            Self::NullPointer(_) => write!(f, "null"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Integer(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Array { items, .. } => {
                write!(f, "[")?;
                let mut items_iter = items.iter();
                if let Some(item) = items_iter.next() {
                    write!(f, " {format} {item}", format = item.format())?;
                    for item in items_iter {
                        write!(f, ", {format} {item}", format = item.format())?;
                    }
                    write!(f, " ")?;
                }
                write!(f, "]")
            },
            Self::Structure { members, .. } => {
                write!(f, "{{")?;
                let mut members_iter = members.iter();
                if let Some(member) = members_iter.next() {
                    write!(f, " {format} {member}", format = member.format())?;
                    for member in members_iter {
                        write!(f, ", {format} {member}", format = member.format())?;
                    }
                    write!(f, " ")?;
                }
                write!(f, "}}")
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
    Break,
    Continue,
    Void,
    Array {
        items: Vec<Value>,
        item_format: Format,
    },
    Structure {
        members: Vec<Value>,
        format: Format,
    },
    Constant(Constant),
    Register(Register),
    Indirect {
        pointer: Box<Value>,
        loaded_format: Format,
    },
    TypeDefinition(Option<Format>),
}

impl Value {
    pub fn format(&self) -> Format {
        match self {
            Self::Never | Self::Break | Self::Continue => Format::Never,
            Self::Void => Format::Void,
            Self::Array { items, item_format } => Format::Array {
                item_format: Box::new(item_format.clone()),
                length: Some(items.len()),
            },
            Self::Structure { format, .. } => format.clone(),
            Self::Constant(constant) => constant.format(),
            Self::Register(register) => register.format().clone(),
            Self::Indirect { loaded_format, .. } => loaded_format.clone(),
            Self::TypeDefinition(_) => Format::Type,
        }
    }

    pub fn into_mutable_lvalue(self) -> crate::Result<(Self, Format)> {
        match self {
            Self::Indirect { pointer, loaded_format } => {
                if loaded_format.is_mutable() {
                    Ok((*pointer, loaded_format))
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
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never | Self::Break | Self::Continue => write!(f, "<ERROR never value>"),
            Self::Void => write!(f, "<ERROR void value>"),
            Self::Array { items, .. } => {
                write!(f, "[")?;
                let mut items_iter = items.iter();
                if let Some(item) = items_iter.next() {
                    write!(f, " {format} {item}", format = item.format())?;
                    for item in items_iter {
                        write!(f, ", {format} {item}", format = item.format())?;
                    }
                    write!(f, " ")?;
                }
                write!(f, "]")
            },
            Self::Structure { members, .. } => {
                write!(f, "{{")?;
                let mut members_iter = members.iter();
                if let Some(member) = members_iter.next() {
                    write!(f, " {format} {member}", format = member.format())?;
                    for member in members_iter {
                        write!(f, ", {format} {member}", format = member.format())?;
                    }
                    write!(f, " ")?;
                }
                write!(f, "}}")
            },
            Self::Constant(constant) => write!(f, "{constant}"),
            Self::Register(register) => write!(f, "{register}"),
            Self::Indirect { .. } => write!(f, "<ERROR indirect value>"),
            Self::TypeDefinition(_) => write!(f, "<ERROR type value>"),
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

    pub fn get_identified_format(&self, type_name: &str) -> Format {
        Format::Identified {
            identifier: format!("type.{type_name}"),
            type_name: type_name.into(),
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

    pub fn get_definition_format(&self, type_name: &str) -> crate::Result<Format> {
        if let Some(symbol) = self.global_symbol_table.find(type_name) {
            if let Value::TypeDefinition(format) = symbol.value() {
                if let Some(format) = format {
                    Ok(format.clone())
                }
                else {
                    Err(crate::RawError::new(format!("type '{type_name}' is declared but not defined")).into_boxed())
                }
            }
            else {
                Err(crate::RawError::new(format!("cannot use '{type_name}' as a type")).into_boxed())
            }
        }
        else {
            Err(crate::RawError::new(format!("unrecognized type name '{type_name}'")).into_boxed())
        }
    }
    
    pub fn get_format_from_node(&self, type_node: &ast::TypeNode, allow_mutable: bool, allow_unsized: bool) -> crate::Result<Format> {
        let format = match type_node {
            ast::TypeNode::Named(type_name) => {
                match type_name.as_str() {
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
                    type_name => self.get_identified_format(type_name)
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
            format.expect_size(&self.global_symbol_table)?;
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
            ast::Node::Literal(literal) => {
                self.generate_literal(literal, context, expected_format.as_ref())?
            },
            ast::Node::Unary { operation, operand } => {
                self.generate_unary_operation(*operation, operand.as_ref(), context, expected_format.as_ref())?
            },
            ast::Node::Binary { operation, lhs, rhs } => {
                self.generate_binary_operation(*operation, lhs.as_ref(), rhs.as_ref(), context, expected_format.as_ref())?
            },
            ast::Node::Call { callee, arguments } => {
                self.generate_call_operation(callee.as_ref(), arguments, context)?
            },
            ast::Node::Array { items } => {
                self.generate_array_literal(items, context, expected_format.as_ref())?
            },
            ast::Node::Structure { type_name, members } => {
                self.generate_structure_literal(type_name.as_ref(), members, context)?
            },
            ast::Node::Scope { statements } => {
                self.local_symbol_table.enter_scope();

                let mut result = Value::Void;
                for statement in statements {
                    let statement_value = self.generate_node(statement.as_ref(), context, None)?;

                    if statement_value.format() == Format::Never {
                        // The rest of the statements in the block will never be executed, so they don't need to be generated
                        result = statement_value;
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
                    if consequent_value.format() != Format::Never {
                        let tail_label = tail_label.get_or_insert_with(|| self.new_anonymous_label());
                        self.emitter.emit_unconditional_branch(tail_label)?;
                    }

                    self.emitter.emit_label(&alternative_label)?;
                    let alternative_value = self.generate_node(alternative.as_ref(), context, None)?;
                    if alternative_value.format() != Format::Never {
                        let tail_label = tail_label.get_or_insert_with(|| self.new_anonymous_label());
                        self.emitter.emit_unconditional_branch(tail_label)?;
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
                    let consequent_value = self.generate_node(consequent.as_ref(), context, None)?;
                    if consequent_value.format() != Format::Never {
                        self.emitter.emit_unconditional_branch(&alternative_label)?;
                    }

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
                // self.next_anonymous_register_id += 1;

                Value::Break
            },
            ast::Node::Continue => {
                let continue_label = context.continue_label()
                    .ok_or_else(|| crate::RawError::new(String::from("unexpected 'continue' outside loop")).into_boxed())?;

                self.emitter.emit_unconditional_branch(continue_label)?;

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                // self.next_anonymous_register_id += 1;

                Value::Continue
            },
            ast::Node::Return { value } => {
                let return_format = context.function().map(|function| function.return_format())
                    .ok_or_else(|| crate::RawError::new(String::from("'return' outside of function")).into_boxed())?;

                if let Some(value) = value {
                    if return_format == &Format::Void {
                        return Err(crate::RawError::new(String::from("returning without a value from a non-void function")).into_boxed());
                    }
                    else {
                        let value = self.generate_node(value.as_ref(), context, Some(return_format.clone()))?;
                        let value = self.coerce_to_rvalue(value)?;

                        self.emitter.emit_return(Some(&value))?;
                    }
                }
                else {
                    if return_format == &Format::Void {
                        self.emitter.emit_return(None)?;
                    }
                    else {
                        return Err(crate::RawError::new(String::from("returning with a value from a void function")).into_boxed());
                    }
                }

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                // self.next_anonymous_register_id += 1;

                Value::Never
            },
            ast::Node::Let { name, value_type, value } => {
                self.generate_let_statement(name, value_type, value.as_deref(), context)?
            },
            ast::Node::Constant { name, value_type, value } => {
                self.generate_let_constant_statement(name, value_type, value.as_ref(), context)?
            },
            ast::Node::Function { name, parameters, is_varargs, return_type, body: None } => {
                self.generate_function_declaration(name, parameters, *is_varargs, return_type, context)?
            },
            ast::Node::Function { name, parameters, is_varargs, return_type, body: Some(body) } => {
                self.generate_function_definition(name, parameters, *is_varargs, return_type, body.as_ref(), context)?
            },
            ast::Node::StructureDefinition { name, members: None } => {
                self.generate_structure_declaration(name, context)?
            },
            ast::Node::StructureDefinition { name, members: Some(members) } => {
                self.generate_structure_definition(name, members, context)?
            },
            _ => {
                return Err(crate::RawError::new(format!("unexpected expression: {node}")).into_boxed());
            }
        };

        if let Some(expected_format) = &expected_format {
            self.enforce_format(result, expected_format)
        }
        else {
            Ok(result)
        }
    }

    fn generate_literal(&mut self, literal: &token::Literal, _context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<Value> {
        let result = match literal {
            token::Literal::Identifier(name) => {
                self.get_symbol(name)?.value().clone()
            },
            token::Literal::Integer(value) => {
                let format = expected_format.cloned().unwrap_or(Format::Integer { size: 4, signed: true });
                let value = IntegerValue::new(*value, &format)
                    .ok_or_else(|| crate::RawError::new(format!("'{value}' cannot be used as a value of type '{}'", format.rich_name())).into_boxed())?;
                Value::Constant(Constant::Integer(value))
            },
            token::Literal::Boolean(value) => {
                Value::Constant(Constant::Boolean(*value))
            },
            token::Literal::NullPointer => {
                Value::Constant(Constant::NullPointer(expected_format.cloned().unwrap_or_else(Format::opaque_pointer)))
            },
            token::Literal::String(value) => {
                let constant = Constant::String(value.clone());
                let pointer = self.new_anonymous_constant(constant.format());

                self.emitter.emit_anonymous_constant(&pointer, &constant)?;

                Value::Constant(Constant::Register(pointer))
            },
        };

        Ok(result)
    }

    fn generate_array_literal(&mut self, items: &[Box<ast::Node>], context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<Value> {
        let result = if let Some(array_format) = expected_format {
            if let Format::Array { item_format, .. } = array_format {
                let mut non_constant_items = Vec::new();

                let constant_items: Vec<Constant> = Result::from_iter(items.iter().enumerate().map(|(index, item)| {
                    let item_value = self.generate_node(item.as_ref(), context, Some(item_format.as_ref().clone()))?;

                    if let Value::Constant(item_constant) = item_value {
                        Ok(item_constant)
                    }
                    else {
                        let item_value = self.coerce_to_rvalue(item_value)?;
                        non_constant_items.push((index, item_value));

                        Ok(Constant::Undefined(item_format.as_ref().clone()))
                    }
                }))?;

                let array_pointer = self.new_anonymous_register(array_format.clone().into_pointer());
                let initial_value = Value::Constant(Constant::Array {
                    items: constant_items,
                    item_format: item_format.as_ref().clone(),
                });

                self.emitter.emit_local_allocation(&array_pointer, array_format)?;

                let array_pointer = Value::Register(array_pointer);

                self.emitter.emit_store(&initial_value, &array_pointer)?;
                
                for (index, member) in non_constant_items {
                    let item_pointer = self.new_anonymous_register(member.format().into_pointer());
                    let zero = Value::Constant(Constant::Integer(IntegerValue::Signed32(0)));
                    let index = Value::Constant(Constant::Integer(IntegerValue::Unsigned64(index as u64)));

                    self.emitter.emit_get_element_pointer(&item_pointer, array_format, &array_pointer, &[zero, index])?;
                    self.emitter.emit_store(&member, &Value::Register(item_pointer))?;
                }

                Value::Indirect {
                    pointer: Box::new(array_pointer),
                    loaded_format: array_format.clone(),
                }
            }
            else {
                return Err(crate::RawError::new(String::from("unknown array format")).into_boxed());
            }
        }
        else {
            // TODO
            return Err(crate::RawError::new(String::from("unknown array format")).into_boxed());
        };

        Ok(result)
    }

    fn generate_structure_literal(&mut self, type_name: &ast::Node, initializer_members: &[(String, Box<ast::Node>)], context: &ScopeContext) -> crate::Result<Value> {
        let definition = self.generate_node(type_name, context, Some(Format::Type))?;

        let result = if let Value::TypeDefinition(Some(structure_format)) = definition {
            if let Format::Structure { type_name, members } = &structure_format {
                let type_name = type_name.as_ref().unwrap();
                let result_format = self.get_identified_format(type_name);

                let mut initializer_members = initializer_members.to_vec();
                let mut non_constant_members = Vec::new();

                let constant_members: Vec<Constant> = Result::from_iter(members.iter().enumerate().map(|(index, (member_name, member_format))| {
                    let initializer_index = initializer_members.iter().position(|(name, _)| member_name == name)
                        .ok_or_else(|| crate::RawError::new(format!("missing member '{member_name}' in initializer of struct '{type_name}'")).into_boxed())?;

                    let (_, member_value) = &initializer_members[initializer_index];
                    let member_value = self.generate_node(member_value.as_ref(), context, Some(member_format.clone()))?;

                    initializer_members.swap_remove(initializer_index);

                    if let Value::Constant(member_constant) = member_value {
                        Ok(member_constant)
                    }
                    else {
                        let member_value = self.coerce_to_rvalue(member_value)?;
                        non_constant_members.push((index, member_value));

                        Ok(Constant::Undefined(member_format.clone()))
                    }
                }))?;

                if !initializer_members.is_empty() {
                    return Err(crate::RawError::new(format!("extraneous members for struct '{type_name}'")).into_boxed());
                }

                let structure_pointer = self.new_anonymous_register(result_format.clone().into_pointer());
                let initial_value = Value::Constant(Constant::Structure {
                    members: constant_members,
                    format: result_format.clone(),
                });

                self.emitter.emit_local_allocation(&structure_pointer, &result_format)?;

                let structure_pointer = Value::Register(structure_pointer);

                self.emitter.emit_store(&initial_value, &structure_pointer)?;
                
                for (index, member) in non_constant_members {
                    let member_pointer = self.new_anonymous_register(member.format().into_pointer());
                    let zero = Value::Constant(Constant::Integer(IntegerValue::Signed32(0)));
                    let index = Value::Constant(Constant::Integer(IntegerValue::Signed32(index as i32)));

                    self.emitter.emit_get_element_pointer(&member_pointer, &result_format, &structure_pointer, &[zero, index])?;
                    self.emitter.emit_store(&member, &Value::Register(member_pointer))?;
                }

                Value::Indirect {
                    pointer: Box::new(structure_pointer),
                    loaded_format: result_format,
                }
            }
            else {
                return Err(crate::RawError::new(format!("invalid type definition for '{type_name}'")).into_boxed());
            }
        }
        else {
            return Err(crate::RawError::new(format!("type '{type_name}' has been declared but not defined")).into_boxed());
        };

        Ok(result)
    }

    fn generate_unary_operation(&mut self, operation: ast::UnaryOperation, operand: &ast::Node, context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<Value> {
        let result = match operation {
            ast::UnaryOperation::PostIncrement => todo!(),
            ast::UnaryOperation::PostDecrement => todo!(),
            ast::UnaryOperation::PreIncrement => todo!(),
            ast::UnaryOperation::PreDecrement => todo!(),
            ast::UnaryOperation::Positive => {
                let operand = self.generate_node(operand, context, expected_format.cloned())?;

                self.coerce_to_rvalue(operand)?
            },
            ast::UnaryOperation::Negative => {
                let operand = self.generate_node(operand, context, expected_format.cloned())?;
                let operand = self.coerce_to_rvalue(operand)?;
                let result = self.new_anonymous_register(expected_format.cloned().unwrap_or(operand.format()));

                self.emitter.emit_negation(&result, &operand)?;

                Value::Register(result)
            },
            ast::UnaryOperation::BitwiseNot => {
                let operand = self.generate_node(operand, context, expected_format.cloned())?;
                let operand = self.coerce_to_rvalue(operand)?;
                let result = self.new_anonymous_register(expected_format.cloned().unwrap_or(operand.format()));

                self.emitter.emit_inversion(&result, &operand)?;

                Value::Register(result)
            },
            ast::UnaryOperation::LogicalNot => {
                let operand = self.generate_node(operand, context, Some(Format::Boolean))?;
                let operand = self.coerce_to_rvalue(operand)?;
                let result = self.new_anonymous_register(Format::Boolean);

                self.emitter.emit_inversion(&result, &operand)?;

                Value::Register(result)
            },
            ast::UnaryOperation::Reference => {
                let operand = self.generate_node(operand, context, None)?;

                if let Value::Indirect { pointer, .. } = operand {
                    *pointer
                }
                else {
                    return Err(crate::RawError::new(String::from("operand of '&' must be an lvalue")).into_boxed());
                }
            },
            ast::UnaryOperation::Dereference => {
                let operand = self.generate_node(operand, context, expected_format.cloned().map(Format::into_pointer))?;
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
                if let ast::Node::Type(type_node) = operand {
                    let format = self.get_format_from_node(type_node, true, false)?;
                    let size = format.expect_size(&self.global_symbol_table)?;

                    Value::Constant(Constant::Integer(IntegerValue::Unsigned64(size as u64)))
                }
                else {
                    return Err(crate::RawError::new(String::from("invalid operand for 'sizeof'")).into_boxed());
                }
            },
            ast::UnaryOperation::GetAlign => todo!(),
        };

        Ok(result)
    }

    fn generate_binary_operation(&mut self, operation: ast::BinaryOperation, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<Value> {
        let result = match operation {
            ast::BinaryOperation::Subscript => {
                self.generate_subscript_operation(lhs, rhs, context)?
            },
            ast::BinaryOperation::Access => {
                let structure = self.generate_node(lhs, context, None)?;

                self.generate_member_access(structure, rhs, context)?
            },
            ast::BinaryOperation::DerefAccess => {
                let pointer = self.generate_node(lhs, context, None)?;
                let pointer = self.coerce_to_rvalue(pointer)?;
                let pointer_format = pointer.format();

                if let Format::Pointer(pointee_format) = pointer_format.as_unqualified() {
                    let structure = Value::Indirect {
                        pointer: Box::new(pointer),
                        loaded_format: pointee_format.as_ref().clone(),
                    };

                    self.generate_member_access(structure, rhs, context)?
                }
                else {
                    return Err(crate::RawError::new(format!("cannot dereference value of type '{}'", pointer_format.rich_name())).into_boxed());
                }
            },
            ast::BinaryOperation::Convert => {
                if let ast::Node::Type(type_node) = rhs {
                    let value = self.generate_node(lhs, context, None)?;
                    let value = self.coerce_to_rvalue(value)?;
                    
                    let target_format = self.get_format_from_node(type_node, false, false)?;

                    self.change_format(value, &target_format)?
                }
                else {
                    return Err(crate::RawError::new(String::from("expected a type following 'as'")).into_boxed());
                }
            },
            ast::BinaryOperation::Add => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_addition(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Subtract => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_subtraction(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Multiply => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_multiplication(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Divide => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_division(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Remainder => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_remainder(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::ShiftLeft => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_shift_left(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::ShiftRight => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_shift_right(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::BitwiseAnd => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_and(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::BitwiseOr => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_or(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::BitwiseXor => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_xor(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Equal => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_equal(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::NotEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_not_equal(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::LessThan => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_less_than(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::LessEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_less_equal(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::GreaterThan => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_greater_than(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::GreaterEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_greater_equal(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::LogicalAnd => todo!(),
            ast::BinaryOperation::LogicalOr => todo!(),
            ast::BinaryOperation::Assign => {
                let lhs = self.generate_node(lhs, context, expected_format.cloned())?;
                let (pointer, loaded_format) = lhs.into_mutable_lvalue()?;
                let rhs = self.generate_node(rhs, context, Some(loaded_format))?;
                let rhs = self.coerce_to_rvalue(rhs)?;

                self.emitter.emit_store(&rhs, &pointer)?;

                rhs
            },
            ast::BinaryOperation::MultiplyAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_multiplication(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::DivideAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_division(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::RemainderAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_remainder(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::AddAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_addition(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::SubtractAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_subtraction(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::ShiftLeftAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_shift_left(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::ShiftRightAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_shift_right(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::BitwiseAndAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_and(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::BitwiseXorAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_xor(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::BitwiseOrAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_or(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
        };

        Ok(result)
    }

    fn generate_subscript_operation(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext) -> crate::Result<Value> {
        let lhs = self.generate_node(lhs, context, None)?;
        let rhs = self.generate_node(rhs, context, None)?;
        let rhs = self.coerce_to_rvalue(rhs)?;
        
        if !(matches!(rhs.format().as_unqualified(), Format::Integer { .. })) {
            return Err(crate::RawError::new(String::from("expected an integer index")).into_boxed());
        }

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

                        Ok(Value::Constant(Constant::Indirect {
                            pointer: Box::new(element_pointer),
                            loaded_format: item_format.as_ref().clone(),
                        }))
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

                            Ok(Value::Indirect {
                                pointer: Box::new(Value::Register(element_pointer)),
                                loaded_format: element_format,
                            })
                        },
                        _ => Err(cannot_index_error())
                    },
                    _ => Err(cannot_index_error())
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

                            Ok(Value::Constant(Constant::Indirect {
                                pointer: Box::new(element_pointer),
                                loaded_format: item_format.as_ref().clone(),
                            }))
                        },
                        _ => Err(cannot_index_error())
                    },
                    _ => Err(cannot_index_error())
                },
                _ => Err(cannot_index_error())
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

                        Ok(Value::Indirect {
                            pointer: Box::new(Value::Register(element_pointer)),
                            loaded_format: item_format.as_ref().clone(),
                        })
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

                            Ok(Value::Indirect {
                                pointer: Box::new(Value::Register(element_pointer)),
                                loaded_format: element_format,
                            })
                        },
                        _ => Err(cannot_index_error())
                    },
                    _ => Err(cannot_index_error())
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

                            Ok(Value::Indirect {
                                pointer: Box::new(Value::Register(element_pointer)),
                                loaded_format: item_format.as_ref().clone(),
                            })
                        },
                        _ => Err(cannot_index_error())
                    },
                    _ => Err(cannot_index_error())
                },
                _ => Err(cannot_index_error())
            }
        }
    }

    fn generate_member_access(&mut self, structure: Value, member_name: &ast::Node, _context: &ScopeContext) -> crate::Result<Value> {
        let structure_format = structure.format();

        let cannot_access_error = || crate::RawError::new(format!(
            "cannot access members of non-struct type '{format}'",
            format = structure_format.rich_name(),
        )).into_boxed();

        if let ast::Node::Literal(token::Literal::Identifier(member_name)) = member_name {
            match structure {
                Value::Indirect { pointer, loaded_format } => match loaded_format.as_unqualified() {
                    Format::Identified { type_name, .. } => match self.get_definition_format(type_name)?.as_unqualified() {
                        Format::Structure { members, .. } => {
                            let member_index = members.iter().position(|(name, _)| name == member_name)
                                .ok_or_else(|| crate::RawError::new(format!("member '{member_name}' does not exist in struct '{type_name}'")).into_boxed())?;
                            let member_format = match &loaded_format {
                                Format::Mutable(_) => members[member_index].1.clone().into_mutable(),
                                _ => members[member_index].1.clone()
                            };
                            let member_pointer = self.new_anonymous_register(member_format.clone().into_pointer());
                            let indices = &[
                                Value::Constant(Constant::Integer(IntegerValue::Signed32(0))),
                                Value::Constant(Constant::Integer(IntegerValue::Signed32(member_index as i32))),
                            ];

                            self.emitter.emit_get_element_pointer(
                                &member_pointer,
                                &loaded_format,
                                pointer.as_ref(),
                                indices,
                            )?;

                            Ok(Value::Indirect {
                                pointer: Box::new(Value::Register(member_pointer)),
                                loaded_format: member_format,
                            })
                        },
                        _ => Err(cannot_access_error())
                    },
                    _ => Err(cannot_access_error())
                },
                _ => Err(cannot_access_error())
            }
        }
        else {
            return Err(crate::RawError::new(String::from("expected a struct member name")).into_boxed());
        }
    }

    fn generate_binary_arithmetic_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<(Register, Value, Value)> {
        let lhs = self.generate_node(lhs, context, expected_format.cloned())?;
        let lhs = self.coerce_to_rvalue(lhs)?;

        let rhs = self.generate_node(rhs, context, Some(lhs.format()))?;
        let rhs = self.coerce_to_rvalue(rhs)?;

        let result = self.new_anonymous_register(expected_format.cloned().unwrap_or_else(|| lhs.format()));

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

    fn generate_assignment_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<(Register, Value, Value, Value)> {
        let lhs = self.generate_node(lhs, context, expected_format.cloned())?;
        let (pointer, loaded_format) = lhs.into_mutable_lvalue()?;

        let rhs = self.generate_node(rhs, context, Some(loaded_format.clone()))?;
        let rhs = self.coerce_to_rvalue(rhs)?;

        let lhs = self.new_anonymous_register(loaded_format.clone());
        let result = self.new_anonymous_register(loaded_format);

        self.emitter.emit_load(&lhs, &pointer)?;

        Ok((result, pointer, Value::Register(lhs), rhs))
    }

    fn generate_call_operation(&mut self, callee: &ast::Node, arguments: &[Box<ast::Node>], context: &ScopeContext) -> crate::Result<Value> {
        let callee = self.generate_node(callee, context, None)?;
        let callee = self.coerce_to_rvalue(callee)?;

        let result = if let Format::Function { return_format, parameters, is_varargs, .. } = callee.format() {
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
        };

        Ok(result)
    }

    fn generate_let_statement(&mut self, name: &str, value_type: &ast::TypeNode, value: Option<&ast::Node>, context: &ScopeContext) -> crate::Result<Value> {
        let format = self.get_format_from_node(value_type, true, false)?;
        let (symbol, pointer) = self.get_symbol_table(context).create_indirect_symbol(name.into(), format.clone());

        if context.is_global() {
            let init_value = if let Some(node) = value {
                self.generate_constant_node(node, context, Some(format.clone()))?
            }
            else {
                Constant::ZeroInitializer(format.clone())
            };

            self.emitter.emit_global_allocation(&pointer, &init_value, false)?;
        }
        else {
            self.emitter.emit_local_allocation(&pointer, &format)?;

            if let Some(node) = value {
                let value = self.generate_node(node, context, Some(format.clone()))?;
                let value = self.coerce_to_rvalue(value)?;

                self.emitter.emit_store(&value, &Value::Register(pointer))?;
            }
        }

        self.define_symbol(context, symbol);

        Ok(Value::Void)
    }

    fn generate_let_constant_statement(&mut self, name: &str, value_type: &ast::TypeNode, value: &ast::Node, context: &ScopeContext) -> crate::Result<Value> {
        let format = self.get_format_from_node(value_type, false, false)?;

        if let Some(function) = context.function() {
            let constant = self.generate_constant_node(value, context, Some(format.clone()))?;
            let (symbol, pointer) = self.get_symbol_table(context).create_indirect_local_constant_symbol(name.into(), format.clone(), function.name());

            self.emitter.emit_global_allocation(&pointer, &constant, true)?;
            
            self.define_symbol(context, symbol);
        }
        else {
            let constant = self.generate_constant_node(value, context, Some(format.clone()))?;
            let (symbol, pointer) = self.global_symbol_table.create_indirect_symbol(name.into(), format.clone());

            self.emitter.emit_global_allocation(&pointer, &constant, true)?;

            self.define_symbol(context, symbol);
        }

        Ok(Value::Void)
    }

    fn generate_function_declaration(&mut self, name: &str, parameters: &[(String, ast::TypeNode)], is_varargs: bool, return_type: &ast::TypeNode, context: &ScopeContext) -> crate::Result<Value> {
        // TODO: maybe do multiple passes of the source file to avoid the need for forward declarations
        let return_format = self.get_format_from_node(return_type, false, false)?;
        let parameter_formats: Vec<Format> = Result::from_iter(parameters.iter().map(|(_, parameter_type)| {
            self.get_format_from_node(parameter_type, true, false)
        }))?;

        let format = Format::Function {
            is_defined: false,
            return_format: Box::new(return_format.clone()),
            parameters: parameter_formats.clone(), 
            is_varargs,
        };

        let function_register;
        if let Some(previous_symbol) = self.global_symbol_table.find(name) {
            if let Format::Function {
                return_format: previous_return_format,
                parameters: previous_parameters,
                is_varargs: previous_varargs,
                ..
            } = previous_symbol.format() {
                if previous_return_format.as_ref() == &return_format && &previous_parameters == &parameter_formats && previous_varargs == is_varargs {
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
            let (symbol, register) = self.get_symbol_table(context).create_register_symbol(name.into(), format);

            self.define_symbol(context, symbol);
            function_register = register;
        }

        self.emitter.queue_function_declaration(&function_register, &return_format, &parameter_formats, is_varargs);

        Ok(Value::Void)
    }

    fn generate_function_definition(&mut self, name: &str, parameters: &[(String, ast::TypeNode)], is_varargs: bool, return_type: &ast::TypeNode, body: &ast::Node, context: &ScopeContext) -> crate::Result<Value> {
        let return_format = self.get_format_from_node(return_type, false, false)?;
        let parameter_formats: Vec<Format> = Result::from_iter(parameters.iter().map(|(_, parameter_type)| {
            self.get_format_from_node(parameter_type, true, false)
        }))?;
        
        let parameter_names_iter = parameters.iter().map(|(name, _)| name.clone());

        self.local_symbol_table.clear();
        self.next_anonymous_register_id = 0;
        self.next_anonymous_label_id = 0;
        let function_context = context.clone().enter_function(name.into(), return_format.clone());

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
            is_varargs,
        };

        let function_register;
        if let Some(previous_symbol) = self.global_symbol_table.find_mut(name) {
            if let Format::Function {
                is_defined: already_defined,
                return_format: previous_return_format,
                parameters: previous_parameters,
                is_varargs: previous_varargs,
            } = previous_symbol.format() {
                if previous_return_format.as_ref() == &return_format && &previous_parameters == &parameter_formats && previous_varargs == is_varargs {
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
            let (symbol, register) = self.get_symbol_table(context).create_register_symbol(name.into(), format);

            self.define_symbol(context, symbol);
            function_register = register;
        }

        let input_registers: Vec<_> = parameter_handles.iter()
            .map(|(register, _, _)| register.clone())
            .collect();
        self.emitter.emit_function_enter(&function_register, &return_format, &input_registers, is_varargs)?;
        let entry_label = self.new_anonymous_label();
        self.emitter.emit_label(&entry_label)?;

        for (input_register, symbol, pointer) in parameter_handles {
            self.emitter.emit_local_allocation(&pointer, input_register.format())?;
            self.emitter.emit_store(&Value::Register(input_register), &Value::Register(pointer))?;
            self.define_symbol(&function_context, symbol);
        }

        let body_result = self.generate_node(body, &function_context, None)?;

        if body_result.format() != Format::Never {
            if &return_format == &Format::Void {
                self.emitter.emit_return(None)?;
            }
            else {
                return Err(crate::RawError::new(format!("non-void function '{name}' could finish without returning a value")).into_boxed());
            }
        }

        self.emitter.emit_function_exit()?;

        Ok(Value::Void)
    }

    fn generate_structure_declaration(&mut self, name: &str, context: &ScopeContext) -> crate::Result<Value> {
        if let Some(previous_symbol) = self.global_symbol_table.find(name) {
            if let Value::TypeDefinition(_) = previous_symbol.value() {
                Ok(Value::Void)
            }
            else {
                return Err(crate::RawError::new(format!("struct '{name}' conflicts with existing global name")).into_boxed());
            }
        }
        else {
            self.emitter.queue_type_declaration(&format!("type.{name}"));

            let symbol = self.global_symbol_table.create_type_definition_symbol(name.into(), None);
            self.define_symbol(context, symbol);

            Ok(Value::Void)
        }
    }

    fn generate_structure_definition(&mut self, name: &str, members: &[(String, ast::TypeNode)], context: &ScopeContext) -> crate::Result<Value> {
        if let Some(previous_symbol) = self.global_symbol_table.find(name) {
            if let Value::TypeDefinition(definition_format) = previous_symbol.value() {
                if definition_format.is_some() {
                    return Err(crate::RawError::new(format!("a type named '{name}' has already been defined")).into_boxed());
                }
            }
            else {
                return Err(crate::RawError::new(format!("struct '{name}' conflicts with existing global name")).into_boxed());
            }
        }

        // Declare the symbol to allow recursive structure definitions
        let symbol = self.global_symbol_table.create_type_definition_symbol(name.into(), None);
        self.define_symbol(context, symbol);

        let members: Vec<(String, Format)> = Result::from_iter(members.iter().map(|(member_name, member_type)| {
            let member_format = self.get_format_from_node(member_type, false, false)?;
            Ok((member_name.clone(), member_format))
        }))?;
        
        let identifier = format!("type.{name}");
        let structure_format = Format::Structure {
            type_name: Some(name.into()),
            members,
        };

        self.emitter.emit_type_definition(&identifier, &structure_format)?;

        let symbol = self.global_symbol_table.create_type_definition_symbol(name.into(), Some(structure_format));
        self.define_symbol(context, symbol);

        Ok(Value::Void)
    }

    pub fn generate_constant_node(&mut self, node: &ast::Node, context: &ScopeContext, expected_format: Option<Format>) -> crate::Result<Constant> {
        let mut constant_id = self.next_anonymous_constant_id;
        let (constant, intermediate_constants) = self.fold_as_constant(node, &mut constant_id, context, expected_format)?;
        self.next_anonymous_constant_id = constant_id;

        for (pointer, intermediate_constant) in &intermediate_constants {
            self.emitter.emit_anonymous_constant(pointer, intermediate_constant)?;
        }

        Ok(constant)
    }

    pub fn fold_as_constant(&self, node: &ast::Node, constant_id: &mut usize, context: &ScopeContext, expected_format: Option<Format>) -> crate::Result<(Constant, Vec<(Register, Constant)>)> {
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
            ast::Node::Array { items } => {
                if let Some(Format::Array { item_format, .. }) = &expected_format {
                    let items: Vec<Constant> = Result::from_iter(items.iter().map(|item| {
                        let (item, mut constants) = self.fold_as_constant(item.as_ref(), constant_id, context, Some(item_format.as_ref().clone()))?;

                        intermediate_constants.append(&mut constants);
                        Ok(item)
                    }))?;

                    Constant::Array {
                        items,
                        item_format: item_format.as_ref().clone(),
                    }
                }
                else {
                    // TODO
                    return Err(crate::RawError::new(String::from("unknown array format")).into_boxed());
                }
            },
            ast::Node::Structure { type_name, members: initializer_members } => {
                if let ast::Node::Literal(token::Literal::Identifier(name)) = type_name.as_ref() {
                    if let Value::TypeDefinition(structure_format) = self.get_symbol(name)?.value() {
                        if let Some(Format::Structure { type_name, members }) = structure_format {
                            let type_name = type_name.as_ref().unwrap();
                            let result_format = self.get_identified_format(type_name);

                            let mut initializer_members = initializer_members.clone();

                            let members: Vec<Constant> = Result::from_iter(members.iter().map(|(member_name, member_format)| {
                                let initializer_index = initializer_members.iter().position(|(name, _)| member_name == name)
                                    .ok_or_else(|| crate::RawError::new(format!("missing member '{member_name}' in initializer of struct '{type_name}'")).into_boxed())?;

                                let (_, member_value) = &initializer_members[initializer_index];
                                let (member_value, mut constants) = self.fold_as_constant(member_value.as_ref(), constant_id, context, Some(member_format.clone()))?;

                                intermediate_constants.append(&mut constants);
                                initializer_members.swap_remove(initializer_index);
                                Ok(member_value)
                            }))?;

                            if !initializer_members.is_empty() {
                                return Err(crate::RawError::new(format!("extraneous members for struct '{name}'")).into_boxed());
                            }

                            Constant::Structure {
                                members,
                                format: result_format,
                            }
                        }
                        else {
                            return Err(crate::RawError::new(format!("type '{name}' has been declared but not defined")).into_boxed());
                        }
                    }
                    else {
                        return Err(crate::RawError::new(format!("expected struct name, got '{name}'")).into_boxed());
                    }
                }
                else {
                    return Err(crate::RawError::new(String::from("expected struct name")).into_boxed());
                }
            },
            ast::Node::Binary { operation, lhs, rhs } => match operation {
                ast::BinaryOperation::Convert => {
                    if let ast::Node::Type(type_node) = rhs.as_ref() {
                        let (value, mut constants) = self.fold_as_constant(lhs.as_ref(), constant_id, context, None)?;
                        intermediate_constants.append(&mut constants);
                        
                        let target_format = self.get_format_from_node(type_node, false, false)?;
    
                        if let Constant::Integer(integer) = value {
                            Constant::Integer(IntegerValue::new(integer.expanded_value(), &target_format)
                                .ok_or_else(|| crate::RawError::new(format!(
                                    "cannot convert from {original_format} to {target_format}",
                                    original_format = integer.format().rich_name(),
                                    target_format = target_format.rich_name(),
                                )).into_boxed())?)
                        }
                        else {
                            return Err(crate::RawError::new(String::from("unsupported constant conversion")).into_boxed());
                        }
                    }
                    else {
                        return Err(crate::RawError::new(String::from("expected a type following 'as'")).into_boxed());
                    }
                },
                _ => {
                    return Err(crate::RawError::new(String::from("unexpected operation in constant")).into_boxed());
                }
            },
            _ => {
                return Err(crate::RawError::new(String::from("unexpected expression in constant")).into_boxed());
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

        while let Some(statement) = parser.parse_statement(true, true)? {
            self.generate_node(statement.as_ref(), &global_context, None)?;
        }

        self.emitter.emit_postamble()
    }
}
