use super::*;

use std::fmt;

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
}

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
    Function(Box<FunctionSignature>),
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
                symbol_table.find(type_name)?.type_value()??.size(symbol_table)
            },
            Self::Function { .. } => None,
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
                members.iter()
                    .map(|(_name, format)| format.alignment(symbol_table).unwrap())
                    .max()
            },
            Self::Identified { type_name, .. } => {
                symbol_table.find(type_name)?.type_value()??.alignment(symbol_table)
            },
            Self::Function { .. } => None,
        }
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self, Format::Mutable(_))
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
            Self::Function(signature) => {
                let mut name = String::from("function(");
                let mut parameters_iter = signature.parameter_formats().iter();
                if let Some(parameter) = parameters_iter.next() {
                    name = format!("{name}{parameter}", parameter = parameter.rich_name());
                    for parameter in parameters_iter {
                        name = format!("{name}, {parameter}", parameter = parameter.rich_name());
                    }
                    if signature.is_varargs() {
                        name = format!("{name}, ..");
                    }
                }
                else if signature.is_varargs() {
                    name = format!("{name}..");
                }
                format!("{name}) -> {return_format}", return_format = signature.return_format().rich_name())
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
            Self::Function(signature) => {
                write!(f, "{return_format}(", return_format = signature.return_format())?;
                let mut parameters_iter = signature.parameter_formats().iter();
                if let Some(parameter) = parameters_iter.next() {
                    write!(f, "{parameter}")?;
                    for parameter in parameters_iter {
                        write!(f, ", {parameter}")?;
                    }
                    if signature.is_varargs() {
                        write!(f, ", ...")?;
                    }
                }
                else if signature.is_varargs() {
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
pub enum Symbol {
    Global {
        name: String,
        value: Value,
    },
    Local {
        name: String,
        value: Value,
        scope: Scope,
        version: usize,
    },
    Function {
        name: String,
        register: Register,
        signature: FunctionSignature,
        is_defined: bool,
    },
    Type {
        name: String,
        format: Option<Format>,
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

    pub fn value(&self) -> Option<Value> {
        match self {
            Self::Global { value, .. } => Some(value.clone()),
            Self::Local { value, .. } => Some(value.clone()),
            Self::Function { register, .. } => Some(Value::Register(register.clone())),
            _ => None
        }
    }

    pub fn function_value(&self) -> Option<(&Register, &FunctionSignature, bool)> {
        match self {
            Self::Function { register, signature, is_defined, .. } => Some((register, signature, *is_defined)),
            _ => None
        }
    }

    pub fn type_value(&self) -> Option<Option<&Format>> {
        match self {
            Self::Type { format, .. } => Some(format.as_ref()),
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
        Self {
            hash_table_buckets: hash_table_bins,
            active_scopes: vec![Scope { id: 0 }],
            next_scope_id: 1,
        }
    }

    pub fn capacity(&self) -> usize {
        self.hash_table_buckets.len()
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
            // This is a programmer error and should never happen
            panic!("attempted to leave outermost scope");
        }
    }

    pub fn scope_is_active(&self, scope: &Scope) -> bool {
        self.active_scopes.contains(scope)
    }

    pub fn find(&self, name: &str) -> Option<&Symbol> {
        let index = self.hash_index(name);

        self.find_in_bucket(index, name)
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        let index = self.hash_index(name);

        self.find_in_bucket_mut(index, name)
    }

    pub fn clear_locals(&mut self) {
        for mut current_node_link in self.hash_table_buckets.iter_mut() {
            // FIXME: preferably could avoid the .as_mut().unwrap() with pattern matching but the borrow checker is weird
            // i opened a stackoverflow question about it so we'll see haha
            while current_node_link.is_some() {
                if let Symbol::Local { .. } = current_node_link.as_ref().unwrap().symbol {
                    // Remove the node by replacing the link to the current node with the link to the next node
                    let next_node_link = current_node_link.as_mut().unwrap().next_node.take();
                    *current_node_link = next_node_link;
                    // current_node_link already points to the next node, so it doesn't need to be advanced
                }
                else {
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

    pub fn create_function_symbol(&self, name: String, signature: FunctionSignature, is_defined: bool) -> (Symbol, Register) {
        let register = Register {
            name: name.clone(),
            format: Format::Function(Box::new(signature.clone())),
            is_global: true,
        };
        let symbol = Symbol::Function {
            name,
            register: register.clone(),
            is_defined,
            signature,
        };

        (symbol, register)
    }

    pub fn create_type_definition_symbol(&self, name: String, format: Option<Format>) -> Symbol {
        Symbol::Type {
            name,
            format,
        }
    }

    pub fn create_global_indirect_symbol(&self, name: String, loaded_format: Format) -> (Symbol, Register) {
        let pointer = Register {
            name: name.clone(),
            format: loaded_format.clone().into_pointer(),
            is_global: true,
        };
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(pointer.clone())),
            loaded_format,
        };
        let symbol = Symbol::Global {
            name,
            value,
        };

        (symbol, pointer)
    }

    pub fn create_local_indirect_symbol(&self, name: String, loaded_format: Format) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let version = self.next_local_symbol_version(&name);
        let qualified_name = match version {
            0 => format!("{name}"),
            _ => format!("{name}-{version}"),
        };
        let pointer = Register {
            name: qualified_name,
            format: loaded_format.clone().into_pointer(),
            is_global: false,
        };
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(pointer.clone())),
            loaded_format,
        };
        let symbol = Symbol::Local {
            name,
            value,
            scope,
            version,
        };

        (symbol, pointer)
    }

    pub fn create_indirect_local_constant_symbol(&self, name: String, loaded_format: Format, function_name: &str) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let version = self.next_local_symbol_version(&name);
        let qualified_name = match version {
            0 => format!("{function_name}.{name}"),
            _ => format!("{function_name}.{name}-{version}"),
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
        let symbol = Symbol::Local {
            name,
            value,
            scope,
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

    fn find_in_bucket(&self, index: usize, name: &str) -> Option<&Symbol> {
        let mut next_node = self.hash_table_buckets.get(index)?.as_deref();

        while let Some(current_node) = next_node {
            if let Symbol::Local { name: symbol_name, scope, .. } = &current_node.symbol {
                if symbol_name == name && self.active_scopes.contains(scope) {
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

    fn find_in_bucket_mut(&mut self, index: usize, name: &str) -> Option<&mut Symbol> {
        let mut next_node = self.hash_table_buckets.get_mut(index)?.as_deref_mut();

        while let Some(current_node) = next_node {
            if let Symbol::Local { name: symbol_name, scope, .. } = &current_node.symbol {
                if symbol_name == name && self.active_scopes.contains(scope) {
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