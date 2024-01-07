use super::*;

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

    pub fn size(&self, symbol_table: &SymbolTable) -> Option<usize> {
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

    pub fn alignment(&self, symbol_table: &SymbolTable) -> Option<usize> {
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

    pub fn expect_size(&self, symbol_table: &SymbolTable) -> crate::Result<usize> {
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
    pub fn new_global(name: String, format: Format) -> Self {
        Self {
            name,
            format,
            is_global: true,
        }
    }

    pub fn new_local(name: String, format: Format) -> Self {
        Self {
            name,
            format,
            is_global: false,
        }
    }

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

#[derive(Clone, PartialEq, Debug)]
pub struct Scope {
    id: usize,
}

impl Scope {
    pub fn id(&self) -> usize {
        self.id
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Symbol {
    identifier: String,
    value: Value,
    scope: Scope,
    version: usize,
}

impl Symbol {
    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn version(&self) -> usize {
        self.version
    }

    pub fn format(&self) -> Format {
        self.value.format()
    }
}

#[derive(Clone, Debug)]
struct SymbolTableNode {
    symbol: Symbol,
    next_node: Option<Box<SymbolTableNode>>,
}

#[derive(Debug)]
pub struct SymbolTable {
    is_global: bool,
    hash_table_bins: Vec<Option<SymbolTableNode>>,
    active_scopes: Vec<Scope>,
    next_scope_id: usize,
}

impl SymbolTable {
    pub fn new(capacity: usize, is_global: bool) -> Self {
        let mut hash_table_bins = Vec::new();
        hash_table_bins.resize_with(capacity, Default::default);
        Self {
            is_global,
            hash_table_bins,
            active_scopes: vec![Scope { id: 0 }],
            next_scope_id: 1,
        }
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }

    pub fn capacity(&self) -> usize {
        self.hash_table_bins.len()
    }

    pub fn clear(&mut self) {
        for root_node in self.hash_table_bins.iter_mut() {
            *root_node = None;
        }
    }

    pub fn current_scope(&self) -> &Scope {
        self.active_scopes.last().unwrap()
    }

    pub fn enter_scope(&mut self) {
        let id = self.next_scope_id;
        self.next_scope_id += 1;
        self.active_scopes.push(Scope { id });
    }

    pub fn leave_scope(&mut self) {
        if self.active_scopes.len() > 1 {
            self.active_scopes.pop();
        }
        else {
            panic!("attempted to leave outermost scope");
        }
    }

    pub fn scope_is_active(&self, scope: &Scope) -> bool {
        self.active_scopes.contains(scope)
    }

    pub fn find(&self, identifier: &str) -> Option<&Symbol> {
        let index = self.hash_index(identifier);

        self.find_in_bin(index, identifier, true)
    }

    pub fn find_mut(&mut self, identifier: &str) -> Option<&mut Symbol> {
        let index = self.hash_index(identifier);

        self.find_in_bin_mut(index, identifier, true)
    }

    pub fn next_symbol_version(&self, identifier: &str) -> usize {
        let index = self.hash_index(identifier);

        self.find_in_bin(index, identifier, false)
            .map_or(0, |symbol| symbol.version() + 1)
    }

    pub fn create_register_symbol(&self, identifier: String, format: Format) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let version = self.next_symbol_version(&identifier);
        let qualified_name = if version == 0 {
            identifier.clone()
        } else {
            format!("{identifier}-{version}")
        };
        let register = Register {
            name: qualified_name,
            format,
            is_global: self.is_global(),
        };
        let symbol = Symbol {
            identifier,
            value: Value::Register(register.clone()),
            scope,
            version,
        };

        (symbol, register)
    }

    pub fn create_indirect_symbol(&self, identifier: String, loaded_format: Format) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let version = self.next_symbol_version(&identifier);
        let qualified_name = if version == 0 {
            identifier.clone()
        }
        else {
            format!("{identifier}-{version}")
        };
        let pointer = Register {
            name: qualified_name,
            format: loaded_format.clone().into_pointer(),
            is_global: self.is_global(),
        };
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(pointer.clone())),
            loaded_format,
        };
        let symbol = Symbol {
            identifier,
            value,
            scope,
            version,
        };

        (symbol, pointer)
    }

    pub fn create_indirect_local_constant_symbol(&self, identifier: String, loaded_format: Format, function_name: &str) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let version = self.next_symbol_version(&identifier);
        let qualified_name = if version == 0 {
            format!("{function_name}.{identifier}")
        }
        else {
            format!("{function_name}.{identifier}-{version}")
        };
        let pointer = Register {
            name: qualified_name,
            format: loaded_format.clone().into_pointer(),
            is_global: true,
        };
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(pointer.clone())),
            loaded_format,
        };
        let symbol = Symbol {
            identifier,
            value,
            scope,
            version,
        };

        (symbol, pointer)
    }

    pub fn create_type_definition_symbol(&self, identifier: String, definition_format: Option<Format>) -> Symbol {
        let scope = self.current_scope().clone();
        let version = self.next_symbol_version(&identifier);
        let symbol = Symbol {
            identifier,
            value: Value::TypeDefinition(definition_format),
            scope,
            version,
        };

        symbol
    }

    pub fn insert(&mut self, symbol: Symbol) {
        let index = self.hash_index(symbol.identifier());
        
        let root_node = &mut self.hash_table_bins[index];
        let node_to_insert = SymbolTableNode {
            symbol,
            next_node: root_node.take().map(|node| Box::new(node)),
        };
        *root_node = Some(node_to_insert);
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

    fn find_in_bin(&self, index: usize, identifier: &str, check_scope: bool) -> Option<&Symbol> {
        let mut next_node = self.hash_table_bins.get(index)?.as_ref();

        while let Some(current_node) = next_node {
            let is_in_scope = !check_scope || self.active_scopes.contains(current_node.symbol.scope());
            if current_node.symbol.identifier() == identifier && is_in_scope {
                return Some(&current_node.symbol);
            }
            next_node = current_node.next_node.as_deref();
        }

        None
    }

    fn find_in_bin_mut(&mut self, index: usize, identifier: &str, check_scope: bool) -> Option<&mut Symbol> {
        let mut next_node = self.hash_table_bins.get_mut(index)?.as_mut();

        while let Some(current_node) = next_node {
            // I would use self.scope_is_active() here, but that requires an immutable borrow of *self, not just self.active_scopes
            let is_in_scope = !check_scope || self.active_scopes.contains(current_node.symbol.scope());
            if current_node.symbol.identifier() == identifier && is_in_scope {
                return Some(&mut current_node.symbol);
            }
            next_node = current_node.next_node.as_deref_mut();
        }

        None
    }
}