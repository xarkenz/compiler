use super::*;

use std::fmt;

pub use crate::ast::PointerSemantics;

fn quote_identifier_if_needed(mut identifier: String) -> String {
    let needs_quotes = identifier.contains(|ch| !(matches!(ch,
        '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '_' | '.' | '$'
    )));

    if needs_quotes {
        identifier.insert(0, '"');
        identifier.push('"');
    }

    identifier
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FunctionSignature {
    return_format: Format,
    parameter_formats: Vec<Format>,
    is_varargs: bool,
}

impl FunctionSignature {
    pub fn new(return_format: Format, parameter_formats: Vec<Format>, is_varargs: bool) -> Self {
        Self {
            return_format,
            parameter_formats,
            is_varargs,
        }
    }

    pub fn return_format(&self) -> &Format {
        &self.return_format
    }

    pub fn parameter_formats(&self) -> &[Format] {
        &self.parameter_formats
    }

    pub fn is_varargs(&self) -> bool {
        self.is_varargs
    }

    pub fn rich_name(&self) -> String {
        let mut name = String::from("function(");
        let mut parameters_iter = self.parameter_formats().iter();
        if let Some(parameter) = parameters_iter.next() {
            name = format!("{name}{parameter}", parameter = parameter.rich_name());
            for parameter in parameters_iter {
                name = format!("{name}, {parameter}", parameter = parameter.rich_name());
            }
            if self.is_varargs() {
                name = format!("{name}, ..");
            }
        }
        else if self.is_varargs() {
            name = format!("{name}..");
        }
        format!("{name}) -> {return_format}", return_format = self.return_format().rich_name())
    }
}

impl fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{return_format}(", return_format = self.return_format())?;
        let mut parameters_iter = self.parameter_formats().iter();
        if let Some(parameter) = parameters_iter.next() {
            write!(f, "{parameter}")?;
            for parameter in parameters_iter {
                write!(f, ", {parameter}")?;
            }
            if self.is_varargs() {
                write!(f, ", ...")?;
            }
        }
        else if self.is_varargs() {
            write!(f, "...")?;
        }
        write!(f, ")")
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Format {
    Never,
    Void,
    Boolean,
    Integer {
        size: usize,
        signed: bool,
    },
    Pointer {
        pointee_format: Box<Format>,
        semantics: PointerSemantics,
    },
    Array {
        item_format: Box<Format>,
        length: Option<usize>,
    },
    Structure {
        type_name: Option<String>,
        members: Vec<(String, Format)>,
    },
    Identified {
        type_identifier: String,
        type_name: String,
    },
    Function {
        signature: Box<FunctionSignature>,
    },
    Scope,
}

impl Format {
    pub fn opaque_pointer() -> Self {
        Self::Pointer {
            pointee_format: Box::new(Format::Void),
            semantics: PointerSemantics::Immutable,
        }
    }

    pub fn into_pointer(self, semantics: PointerSemantics) -> Self {
        Self::Pointer {
            pointee_format: Box::new(self),
            semantics,
        }
    }

    pub fn size(&self, symbol_table: &SymbolTable) -> Option<usize> {
        match self {
            Self::Never => Some(0),
            Self::Void => Some(0),
            Self::Boolean => Some(1),
            Self::Integer { size, .. } => Some(*size),
            Self::Pointer { .. } => Some(8),
            Self::Array { length, item_format } => {
                Some(length.clone()? * item_format.size(symbol_table)?)
            },
            Self::Structure { members, .. } => {
                let mut current_size = 0;
                let mut max_alignment = 0;

                for (_name, format) in members {
                    let alignment = format.alignment(symbol_table)?;
                    max_alignment = max_alignment.max(alignment);

                    // Calculate padding
                    let intermediate_size = current_size + alignment - 1;
                    let padded_size = intermediate_size - intermediate_size % alignment;
                    current_size = padded_size + format.size(symbol_table)?;
                }

                // Pad for the largest member alignment
                let intermediate_size = current_size + max_alignment - 1;
                let padded_size = intermediate_size - intermediate_size % max_alignment;
                
                Some(padded_size)
            },
            Self::Identified { type_name, .. } => {
                symbol_table.find(type_name, None)?
                    .type_value()?
                    .definition_format()?
                    .size(symbol_table)
            },
            Self::Function { .. } => None,
            Self::Scope => None,
        }
    }

    pub fn alignment(&self, symbol_table: &SymbolTable) -> Option<usize> {
        match self {
            Self::Never => None,
            Self::Void => None,
            Self::Boolean => Some(1),
            Self::Integer { size, .. } => Some(*size),
            Self::Pointer { .. } => Some(8),
            Self::Array { item_format, .. } => item_format.alignment(symbol_table),
            Self::Structure { members, .. } => {
                members.iter()
                    .map(|(_name, format)| format.alignment(symbol_table).unwrap())
                    .max()
            },
            Self::Identified { type_name, .. } => {
                symbol_table.find(type_name, None)?
                    .type_value()?
                    .definition_format()?
                    .alignment(symbol_table)
            },
            Self::Function { .. } => None,
            Self::Scope => None,
        }
    }

    pub fn pointer_semantics(&self) -> Option<PointerSemantics> {
        match self {
            Self::Pointer { semantics, .. } => Some(*semantics),
            _ => None
        }
    }

    pub fn can_coerce_to(&self, other: &Self, is_mutable: bool) -> bool {
        self == other || match (self, other) {
            (Self::Pointer { pointee_format: self_pointee, semantics: self_semantics }, Self::Pointer { pointee_format: other_pointee, semantics: other_semantics }) => {
                // Not sure if this is right... needs testing
                match (self_semantics, other_semantics) {
                    (PointerSemantics::Immutable, PointerSemantics::Immutable) => self_pointee.can_coerce_to(other_pointee, false),
                    (PointerSemantics::Immutable, _) => false,
                    (PointerSemantics::Mutable, PointerSemantics::Immutable) => self_pointee.can_coerce_to(other_pointee, true),
                    (PointerSemantics::Mutable, PointerSemantics::Mutable) => self_pointee.can_coerce_to(other_pointee, true),
                    (PointerSemantics::Mutable, _) => false,
                    (PointerSemantics::Owned, PointerSemantics::Immutable) => self_pointee.can_coerce_to(other_pointee, is_mutable),
                    (PointerSemantics::Owned, PointerSemantics::Mutable) => is_mutable && self_pointee.can_coerce_to(other_pointee, is_mutable),
                    (PointerSemantics::Owned, PointerSemantics::Owned) => self_pointee.can_coerce_to(other_pointee, is_mutable),
                }
            },
            (Self::Array { item_format: self_item, length: Some(self_length) }, Self::Array { item_format: other_item, length: Some(other_length) }) => {
                self_length == other_length && self_item.can_coerce_to(other_item, is_mutable)
            },
            (Self::Array { item_format: self_item, length: _ }, Self::Array { item_format: other_item, length: None }) => {
                self_item.can_coerce_to(other_item, is_mutable)
            },
            (Self::Void, _) | (_, Self::Void) => true,
            _ => false
        }
    }

    pub fn requires_bitcast_to(&self, other: &Self) -> bool {
        self != other && match (self, other) {
            (Self::Pointer { pointee_format: self_pointee, .. }, Self::Pointer { pointee_format: other_pointee, .. }) => {
                self_pointee.requires_bitcast_to(other_pointee)
            },
            (Self::Array { item_format: self_item, length: self_length }, Self::Array { item_format: other_item, length: other_length }) => {
                self_length != other_length || self_item.requires_bitcast_to(other_item)
            },
            _ => true
        }
    }

    pub fn expect_size(&self, symbol_table: &SymbolTable) -> crate::Result<usize> {
        self.size(symbol_table).ok_or_else(|| Box::new(crate::Error::UnknownTypeSize { type_name: self.rich_name() }))
    }

    pub fn rich_name(&self) -> String {
        match self {
            Self::Never => {
                "never".into()
            }
            Self::Void => {
                "void".into()
            },
            Self::Boolean => {
                "bool".into()
            },
            Self::Integer { size, signed: true } => {
                format!("i{bits}", bits = size * 8)
            },
            Self::Integer { size, signed: false } => {
                format!("u{bits}", bits = size * 8)
            },
            Self::Pointer { pointee_format, semantics } => {
                let pointee_format = pointee_format.rich_name();
                match semantics {
                    PointerSemantics::Immutable => format!("*{pointee_format}"),
                    PointerSemantics::Mutable => format!("*mut {pointee_format}"),
                    PointerSemantics::Owned => format!("*own {pointee_format}"),
                }
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
            Self::Function { signature } => {
                signature.rich_name()
            },
            Self::Scope => {
                "{scope}".into()
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
            Self::Integer { size, .. } => {
                write!(f, "i{bits}", bits = size * 8)
            },
            Self::Pointer { pointee_format, .. } => {
                if let Self::Void = pointee_format.as_ref() {
                    write!(f, "{{}}*") // I wanted to use `ptr`, but LLVM complains unless -opaque-pointers is enabled
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
            Self::Identified { type_identifier, .. } => {
                let identifier = quote_identifier_if_needed(format!("type.{type_identifier}"));
                write!(f, "%{identifier}")
            },
            Self::Function { signature } => {
                write!(f, "{signature}")
            },
            Self::Scope => {
                write!(f, "<ERROR scope format>")
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
        match format {
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
    identifier: String,
    format: Format,
    is_global: bool,
}

impl Register {
    pub fn new_global(identifier: String, format: Format) -> Self {
        Self {
            identifier: quote_identifier_if_needed(identifier),
            format,
            is_global: true,
        }
    }

    pub fn new_local(identifier: String, format: Format) -> Self {
        Self {
            identifier: quote_identifier_if_needed(identifier),
            format,
            is_global: false,
        }
    }

    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn format(&self) -> &Format {
        &self.format
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }
}

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.identifier().partial_cmp(other.identifier())
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
            write!(f, "@{}", self.identifier)
        }
        else {
            write!(f, "%{}", self.identifier)
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
        semantics: PointerSemantics,
    },
    Scope(Scope),
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
            Self::GetElementPointer { element_format, semantics, .. } => element_format.clone().into_pointer(*semantics),
            Self::Scope(_) => Format::Scope,
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
            Self::Indirect { pointer, .. } => write!(f, "<ERROR indirect constant: {pointer}>"),
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
            Self::Scope(_) => write!(f, "<ERROR scope constant>"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Never,
    Break,
    Continue,
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
            Self::Never | Self::Break | Self::Continue => Format::Never,
            Self::Void => Format::Void,
            Self::Constant(constant) => constant.format(),
            Self::Register(register) => register.format().clone(),
            Self::Indirect { loaded_format, .. } => loaded_format.clone(),
        }
    }

    pub fn into_mutable_lvalue(self) -> crate::Result<(Self, Format)> {
        match self {
            Self::Indirect { pointer, loaded_format } => {
                if let Some(PointerSemantics::Mutable) = pointer.format().pointer_semantics() {
                    Ok((*pointer, loaded_format))
                }
                else {
                    Err(Box::new(crate::Error::CannotMutateValue { type_name: loaded_format.rich_name() }))
                }
            },
            _ => {
                Err(Box::new(crate::Error::ExpectedLValue {}))
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never | Self::Break | Self::Continue => write!(f, "<ERROR never value>"),
            Self::Void => write!(f, "<ERROR void value>"),
            Self::Constant(constant) => write!(f, "{constant}"),
            Self::Register(register) => write!(f, "{register}"),
            Self::Indirect { pointer, .. } => write!(f, "<ERROR indirect value: {pointer}>"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Label {
    name: String,
}

impl Label {
    pub fn new(name: String) -> Self {
        Self {
            name,
        }
    }

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
pub struct Scope {
    id: usize,
    name: Option<String>,
}

impl Scope {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn get_member_identifier(&self, member_name: &str) -> String {
        let mut member_identifier = self.name()
            .map_or_else(String::new, |name| format!("{name}::"));
        member_identifier.push_str(member_name);
        
        member_identifier
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[derive(Clone, Debug)]
pub struct FunctionSymbol {
    register: Register,
    signature: FunctionSignature,
    is_defined: bool,
}

impl FunctionSymbol {
    pub fn register(&self) -> &Register {
        &self.register
    }

    pub fn signature(&self) -> &FunctionSignature {
        &self.signature
    }

    pub fn is_defined(&self) -> bool {
        self.is_defined
    }

    pub fn value(&self) -> Value {
        Value::Constant(Constant::Register(self.register.clone()))
    }
}

#[derive(Clone, Debug)]
pub struct TypeSymbol {
    definition_format: Option<Format>,
    member_scope: Scope,
}

impl TypeSymbol {
    pub fn definition_format(&self) -> Option<&Format> {
        self.definition_format.as_ref()
    }

    pub fn member_scope(&self) -> &Scope {
        &self.member_scope
    }

    pub fn value(&self) -> Value {
        Value::Constant(Constant::Scope(self.member_scope.clone()))
    }
}

#[derive(Clone, Debug)]
pub enum Symbol {
    Global {
        name: String,
        scope: Scope,
        value: Value,
    },
    Local {
        name: String,
        scope: Scope,
        value: Value,
        version: usize,
    },
    Function {
        name: String,
        scope: Scope,
        content: FunctionSymbol,
    },
    Type {
        name: String,
        scope: Scope,
        content: TypeSymbol,
    },
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Self::Global { name, .. } => name.as_str(),
            Self::Local { name, .. } => name.as_str(),
            Self::Function { name, .. } => name.as_str(),
            Self::Type { name, .. } => name.as_str(),
        }
    }

    pub fn scope(&self) -> &Scope {
        match self {
            Self::Global { scope, .. } => scope,
            Self::Local { scope, .. } => scope,
            Self::Function { scope, .. } => scope,
            Self::Type { scope, .. } => scope,
        }
    }

    pub fn value(&self) -> Value {
        match self {
            Self::Global { value, .. } => value.clone(),
            Self::Local { value, .. } => value.clone(),
            Self::Function { content, .. } => content.value(),
            Self::Type { content, .. } => content.value(),
        }
    }

    pub fn function_value(&self) -> Option<&FunctionSymbol> {
        match self {
            Self::Function { content, .. } => Some(content),
            _ => None
        }
    }

    pub fn type_value(&self) -> Option<&TypeSymbol> {
        match self {
            Self::Type { content, .. } => Some(content),
            _ => None
        }
    }
}

#[derive(Clone, Debug)]
struct SymbolTableNode {
    symbol: Symbol,
    next_node: Option<Box<SymbolTableNode>>,
}

#[derive(Debug)]
pub struct SymbolTable {
    hash_table_buckets: Vec<Option<Box<SymbolTableNode>>>,
    active_scopes: Vec<Scope>,
    next_scope_id: usize,
}

impl SymbolTable {
    pub fn new(capacity: usize) -> Self {
        let mut hash_table_bins = Vec::new();
        hash_table_bins.resize_with(capacity, || None);

        let outermost_scope = Scope {
            id: 0,
            name: None,
        };

        Self {
            hash_table_buckets: hash_table_bins,
            active_scopes: vec![outermost_scope],
            next_scope_id: 1,
        }
    }

    pub fn capacity(&self) -> usize {
        self.hash_table_buckets.len()
    }

    pub fn current_scope(&self) -> &Scope {
        self.active_scopes.last().unwrap()
    }

    pub fn create_inactive_scope(&mut self, name: Option<String>) -> Scope {
        let id = self.next_scope_id;
        self.next_scope_id += 1;

        Scope {
            id,
            name,
        }
    }

    pub fn enter_new_scope(&mut self) {
        let new_scope = self.create_inactive_scope(None);
        self.active_scopes.push(new_scope);
    }

    pub fn enter_scope(&mut self, scope: Scope) {
        self.active_scopes.push(scope);
    }

    pub fn leave_scope(&mut self) {
        if self.active_scopes.len() > 1 {
            self.active_scopes.pop();
        }
        else {
            // This should never occur other than in the case of programmer error
            panic!("attempted to leave outermost scope");
        }
    }

    pub fn scope_is_active(&self, scope: &Scope) -> bool {
        self.active_scopes.contains(scope)
    }

    pub fn find(&self, name: &str, in_scope: Option<&Scope>) -> Option<&Symbol> {
        let index = self.hash_index(name);

        self.find_in_bucket(index, name, in_scope)
    }

    pub fn find_mut(&mut self, name: &str, in_scope: Option<&Scope>) -> Option<&mut Symbol> {
        let index = self.hash_index(name);

        self.find_in_bucket_mut(index, name, in_scope)
    }

    pub fn clear_locals(&mut self) {
        for mut current_node_link in self.hash_table_buckets.iter_mut() {
            // FIXME: preferably could avoid the .as_mut().unwrap() with pattern matching but the borrow checker is weird
            // (As it turns out, the code with pattern matching compiles under the nightly Polonius feature... guess I'm waiting on that then)
            while current_node_link.is_some() {
                if let Symbol::Local { .. } = current_node_link.as_ref().unwrap().symbol {
                    // Remove the node by replacing the link to the current node with the link to the next node
                    let next_node_link = current_node_link.as_mut().unwrap().next_node.take();
                    *current_node_link = next_node_link;
                    // current_node_link already points to the next node, so it doesn't need to be advanced
                }
                else {
                    // Advance current_node_link to the next node
                    current_node_link = &mut current_node_link.as_mut().unwrap().next_node;
                }
            }
        }
    }

    pub fn next_local_symbol_version(&self, name: &str) -> usize {
        let index = self.hash_index(name);

        self.find_local_in_bucket(index, name)
            .map_or(0, |(_, _, version)| version + 1)
    }

    pub fn create_type_symbol(&self, name: String, definition_format: Option<Format>, member_scope: Scope) -> Symbol {
        let scope = self.current_scope().clone();
        let content = TypeSymbol {
            definition_format,
            member_scope,
        };

        Symbol::Type {
            name,
            scope,
            content,
        }
    }

    pub fn create_function_symbol(&self, name: String, signature: FunctionSignature, is_defined: bool) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let identifier = scope.get_member_identifier(&name);
        let format = Format::Function {
            signature: Box::new(signature.clone()),
        };
        let register = Register::new_global(
            identifier,
            format,
        );
        let content = FunctionSymbol {
            register: register.clone(),
            signature,
            is_defined,
        };

        let symbol = Symbol::Function {
            name,
            scope,
            content,
        };

        (symbol, register)
    }

    pub fn create_global_indirect_symbol(&self, name: String, loaded_format: Format, is_mutable: bool) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let identifier = scope.get_member_identifier(&name);
        let semantics = PointerSemantics::simple(is_mutable);
        let pointer = Register::new_global(
            identifier,
            loaded_format.clone().into_pointer(semantics),
        );
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(pointer.clone())),
            loaded_format,
        };

        let symbol = Symbol::Global {
            name,
            scope,
            value,
        };

        (symbol, pointer)
    }

    pub fn create_local_indirect_symbol(&self, name: String, loaded_format: Format, is_mutable: bool) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let version = self.next_local_symbol_version(&name);
        let identifier = match version {
            0 => format!("{name}"),
            _ => format!("{name}-{version}"),
        };
        let semantics = PointerSemantics::simple(is_mutable);
        let pointer = Register::new_local(
            identifier,
            loaded_format.clone().into_pointer(semantics),
        );
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(pointer.clone())),
            loaded_format,
        };

        let symbol = Symbol::Local {
            name,
            scope,
            value,
            version,
        };

        (symbol, pointer)
    }

    pub fn create_indirect_local_constant_symbol(&self, name: String, loaded_format: Format, function_name: &str) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let version = self.next_local_symbol_version(&name);
        let identifier = match version {
            0 => format!("{function_name}.{name}"),
            _ => format!("{function_name}.{name}-{version}"),
        };
        let pointer = Register::new_global(
            identifier,
            loaded_format.clone().into_pointer(PointerSemantics::Immutable),
        );
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(pointer.clone())),
            loaded_format,
        };

        let symbol = Symbol::Local {
            name,
            scope,
            value,
            version,
        };

        (symbol, pointer)
    }

    pub fn insert(&mut self, symbol: Symbol) {
        let index = self.hash_index(symbol.name());
        
        let root_node = &mut self.hash_table_buckets[index];
        let node_to_insert = SymbolTableNode {
            symbol,
            next_node: root_node.take(),
        };
        *root_node = Some(Box::new(node_to_insert));
    }

    fn hash_index(&self, key: &str) -> usize {
        // https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function#FNV_offset_basis
        const FNV_OFFSET_BASIS: u64 = 0xCBF29CE484222325;
        // Any large prime number will do
        const FNV_PRIME: u64 = 0x100000001B3;

        let mut hash = FNV_OFFSET_BASIS;
        for byte_value in key.bytes() {
            hash ^= byte_value as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        hash as usize % self.capacity()
    }

    fn find_in_bucket(&self, index: usize, name: &str, in_scope: Option<&Scope>) -> Option<&Symbol> {
        let mut next_node = self.hash_table_buckets.get(index)?.as_deref();

        while let Some(current_node) = next_node {
            if let Symbol::Local { name: symbol_name, scope, .. } = &current_node.symbol {
                // FIXME: probably need to do more than just self.active_scopes.contains() to ensure the retrieved symbol is from the *closest* scope
                if symbol_name == name && in_scope.map_or_else(|| self.active_scopes.contains(scope), |in_scope| in_scope == scope) {
                    return Some(&current_node.symbol);
                }
            }
            else if current_node.symbol.name() == name {
                return Some(&current_node.symbol);
            }
            next_node = current_node.next_node.as_deref();
        }

        None
    }

    fn find_in_bucket_mut(&mut self, index: usize, name: &str, in_scope: Option<&Scope>) -> Option<&mut Symbol> {
        let mut next_node = self.hash_table_buckets.get_mut(index)?.as_deref_mut();

        while let Some(current_node) = next_node {
            if let Symbol::Local { name: symbol_name, scope, .. } = &current_node.symbol {
                if symbol_name == name && in_scope.map_or_else(|| self.active_scopes.contains(scope), |in_scope| in_scope == scope) {
                    return Some(&mut current_node.symbol);
                }
            }
            else if current_node.symbol.name() == name {
                return Some(&mut current_node.symbol);
            }
            next_node = current_node.next_node.as_deref_mut();
        }

        None
    }

    fn find_local_in_bucket(&self, index: usize, name: &str) -> Option<(&Value, &Scope, usize)> {
        let mut next_node = self.hash_table_buckets.get(index)?.as_deref();

        while let Some(current_node) = next_node {
            if let Symbol::Local { name: symbol_name, value, scope, version } = &current_node.symbol {
                if symbol_name == name {
                    return Some((value, scope, *version));
                }
            }
            next_node = current_node.next_node.as_deref();
        }

        None
    }
}