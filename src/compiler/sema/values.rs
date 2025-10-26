use super::*;

use std::fmt;

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
    pub fn new(raw: i128, type_info: &TypeRepr) -> Option<Self> {
        match type_info {
            TypeRepr::Integer { size: 1, signed: true } => Some(Self::Signed8(raw as i8)),
            TypeRepr::Integer { size: 1, signed: false } => Some(Self::Unsigned8(raw as u8)),
            TypeRepr::Integer { size: 2, signed: true } => Some(Self::Signed16(raw as i16)),
            TypeRepr::Integer { size: 2, signed: false } => Some(Self::Unsigned16(raw as u16)),
            TypeRepr::Integer { size: 4, signed: true } => Some(Self::Signed32(raw as i32)),
            TypeRepr::Integer { size: 4, signed: false } => Some(Self::Unsigned32(raw as u32)),
            TypeRepr::Integer { size: 8, signed: true } => Some(Self::Signed64(raw as i64)),
            TypeRepr::Integer { size: 8, signed: false } => Some(Self::Unsigned64(raw as u64)),
            _ => None
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Signed8(..) | Self::Unsigned8(..) => 1,
            Self::Signed16(..) | Self::Unsigned16(..) => 2,
            Self::Signed32(..) | Self::Unsigned32(..) => 4,
            Self::Signed64(..) | Self::Unsigned64(..) => 8,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Self::Signed8(..) | Self::Signed16(..) | Self::Signed32(..) | Self::Signed64(..) => true,
            Self::Unsigned8(..) | Self::Unsigned16(..) | Self::Unsigned32(..) | Self::Unsigned64(..) => false,
        }
    }

    pub fn expanded_value(&self) -> i128 {
        match *self {
            Self::Signed8(value) => value as i128,
            Self::Unsigned8(value) => value as i128,
            Self::Signed16(value) => value as i128,
            Self::Unsigned16(value) => value as i128,
            Self::Signed32(value) => value as i128,
            Self::Unsigned32(value) => value as i128,
            Self::Signed64(value) => value as i128,
            Self::Unsigned64(value) => value as i128,
        }
    }

    pub fn get_type(&self) -> TypeHandle {
        match self {
            Self::Signed8(..) => TypeHandle::I8,
            Self::Unsigned8(..) => TypeHandle::U8,
            Self::Signed16(..) => TypeHandle::I16,
            Self::Unsigned16(..) => TypeHandle::U16,
            Self::Signed32(..) => TypeHandle::I32,
            Self::Unsigned32(..) => TypeHandle::U32,
            Self::Signed64(..) => TypeHandle::I64,
            Self::Unsigned64(..) => TypeHandle::U64,
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
pub enum FloatValue {
    Float32(f32),
    Float64(f64),
}

impl FloatValue {
    pub fn new(raw: f64, type_info: &TypeRepr) -> Option<Self> {
        match type_info {
            TypeRepr::Float32 => Some(Self::Float32(raw as f32)),
            TypeRepr::Float64 => Some(Self::Float64(raw)),
            _ => None
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Float32(..) => 4,
            Self::Float64(..) => 8,
        }
    }

    pub fn expanded_value(&self) -> f64 {
        match *self {
            Self::Float32(value) => value as f64,
            Self::Float64(value) => value,
        }
    }

    pub fn get_type(&self) -> TypeHandle {
        match self {
            Self::Float32(..) => TypeHandle::F32,
            Self::Float64(..) => TypeHandle::F64,
        }
    }

    pub fn llvm_syntax(&self) -> String {
        // Convert to hexadecimal representation for purposes of keeping exact value
        match *self {
            Self::Float32(value) => format!("0x{:016X}", (value as f64).to_bits()),
            Self::Float64(value) => format!("0x{:016X}", value.to_bits()),
        }
    }
}

impl fmt::Display for FloatValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float32(value) => write!(f, "{value}"),
            Self::Float64(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StringValue {
    bytes: Vec<u8>,
}

impl StringValue {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Display for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c\"")?;
        for &byte in self.bytes() {
            if byte != b'"' && (byte == b' ' || byte.is_ascii_graphic()) {
                write!(f, "{}", byte as char)?;
            }
            else {
                write!(f, "\\{byte:02X}")?;
            }
        }
        write!(f, "\"")
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Register {
    identifier: String,
    value_type: TypeHandle,
    is_global: bool,
}

impl Register {
    pub fn new_global(identifier: String, value_type: TypeHandle) -> Self {
        Self {
            identifier: Self::quote_identifier_if_needed(identifier),
            value_type,
            is_global: true,
        }
    }

    pub fn new_local(identifier: String, value_type: TypeHandle) -> Self {
        Self {
            identifier: Self::quote_identifier_if_needed(identifier),
            value_type,
            is_global: false,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn get_type(&self) -> TypeHandle {
        self.value_type
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }

    pub fn llvm_syntax(&self) -> String {
        if self.is_global() {
            format!("@{}", self.identifier())
        }
        else {
            format!("%{}", self.identifier())
        }
    }

    fn quote_identifier_if_needed(mut identifier: String) -> String {
        let needs_quotes = identifier.contains(|ch| !matches!(ch,
            '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '_' | '.' | '$'
        ));

        if needs_quotes {
            identifier.insert(0, '"');
            identifier.push('"');
        }

        identifier
    }
}

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Register {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.identifier().cmp(other.identifier())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ConversionOperation {
    Truncate,
    ZeroExtend,
    SignExtend,
    FloatTruncate,
    FloatExtend,
    FloatToUnsigned,
    FloatToSigned,
    UnsignedToFloat,
    SignedToFloat,
    PointerToInteger,
    IntegerToPointer,
    BitwiseCast,
}

impl ConversionOperation {
    pub fn from_type_reprs(from_type: &TypeRepr, to_type: &TypeRepr) -> Option<Self> {
        match (from_type, to_type) {
            (
                &TypeRepr::Integer { size: from_size, signed: from_signed },
                &TypeRepr::Integer { size: to_size, .. },
            ) => {
                if from_size > to_size {
                    Some(Self::Truncate)
                }
                else if from_size < to_size {
                    if from_signed {
                        Some(Self::SignExtend)
                    }
                    else {
                        Some(Self::ZeroExtend)
                    }
                }
                else {
                    None
                }
            }
            (TypeRepr::Boolean, TypeRepr::Integer { .. }) => {
                Some(Self::ZeroExtend)
            }
            (TypeRepr::Float64, TypeRepr::Float32) => {
                Some(Self::FloatTruncate)
            }
            (TypeRepr::Float32, TypeRepr::Float64) => {
                Some(Self::FloatExtend)
            }
            (
                TypeRepr::Float32 | TypeRepr::Float64,
                &TypeRepr::Integer { signed, .. },
            ) => {
                if signed {
                    Some(Self::FloatToSigned)
                }
                else {
                    Some(Self::FloatToUnsigned)
                }
            }
            (
                &TypeRepr::Integer { signed, .. },
                TypeRepr::Float32 | TypeRepr::Float64,
            ) => {
                if signed {
                    Some(Self::SignedToFloat)
                }
                else {
                    Some(Self::UnsignedToFloat)
                }
            }
            (TypeRepr::Boolean, TypeRepr::Float32 | TypeRepr::Float64) => {
                Some(Self::UnsignedToFloat)
            }
            (
                TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
                TypeRepr::Integer { .. },
            ) => {
                Some(Self::PointerToInteger)
            }
            (
                TypeRepr::Integer { .. },
                TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
            ) => {
                Some(Self::IntegerToPointer)
            }
            (
                TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
                TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
            ) => {
                Some(Self::BitwiseCast)
            }
            _ => None
        }
    }
}

impl std::fmt::Display for ConversionOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Truncate => write!(f, "trunc"),
            Self::ZeroExtend => write!(f, "zext"),
            Self::SignExtend => write!(f, "sext"),
            Self::FloatTruncate => write!(f, "fptrunc"),
            Self::FloatExtend => write!(f, "fpext"),
            Self::FloatToUnsigned => write!(f, "fptoui"),
            Self::FloatToSigned => write!(f, "fptosi"),
            Self::UnsignedToFloat => write!(f, "uitofp"),
            Self::SignedToFloat => write!(f, "sitofp"),
            Self::PointerToInteger => write!(f, "ptrtoint"),
            Self::IntegerToPointer => write!(f, "inttoptr"),
            Self::BitwiseCast => write!(f, "bitcast"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Constant {
    Undefined(TypeHandle),
    Poison(TypeHandle),
    ZeroInitializer(TypeHandle),
    NullPointer(TypeHandle),
    Boolean(bool),
    Integer(IntegerValue),
    Float(FloatValue),
    String {
        array_type: TypeHandle,
        value: StringValue,
    },
    Array {
        array_type: TypeHandle,
        items: Vec<Constant>,
    },
    Structure {
        struct_type: TypeHandle,
        members: Vec<Constant>,
    },
    Register(Register),
    Indirect {
        pointee_type: TypeHandle,
        pointer: Box<Constant>,
    },
    BitwiseCast {
        result_type: TypeHandle,
        value: Box<Constant>,
    },
    GetElementPointer {
        result_type: TypeHandle,
        aggregate_type: TypeHandle,
        pointer: Box<Constant>,
        indices: Vec<Constant>,
    },
    Type(TypeHandle),
    Module(NamespaceHandle),
}

impl Constant {
    pub fn as_namespace(&self, context: &GlobalContext) -> Option<NamespaceHandle> {
        match *self {
            Self::Type(handle) => Some(context.type_namespace(handle)),
            Self::Module(namespace) => Some(namespace),
            _ => None
        }
    }

    pub fn as_type(&self) -> Option<TypeHandle> {
        match *self {
            Self::Type(handle) => Some(handle),
            _ => None
        }
    }

    pub fn as_module(&self) -> Option<NamespaceHandle> {
        match *self {
            Self::Module(namespace) => Some(namespace),
            _ => None
        }
    }

    pub fn get_type(&self) -> TypeHandle {
        match *self {
            Self::Undefined(value_type) => value_type,
            Self::Poison(value_type) => value_type,
            Self::ZeroInitializer(value_type) => value_type,
            Self::NullPointer(value_type) => value_type,
            Self::Boolean(..) => TypeHandle::BOOL,
            Self::Integer(ref integer) => integer.get_type(),
            Self::Float(ref float) => float.get_type(),
            Self::String { array_type, .. } => array_type,
            Self::Array { array_type, .. } => array_type,
            Self::Structure { struct_type, .. } => struct_type,
            Self::Register(ref register) => register.get_type(),
            Self::Indirect { pointee_type, .. } => pointee_type,
            Self::BitwiseCast { result_type, .. } => result_type,
            Self::GetElementPointer { result_type, .. } => result_type,
            Self::Type(..) | Self::Module(..) => TypeHandle::META,
        }
    }

    pub fn llvm_syntax(&self, context: &GlobalContext) -> String {
        match self {
            Self::Undefined(..) => "undef".to_owned(),
            Self::Poison(..) => "poison".to_owned(),
            Self::ZeroInitializer(..) => "zeroinitializer".to_owned(),
            Self::NullPointer(..) => "null".to_owned(),
            Self::Boolean(value) => format!("{value}"),
            Self::Integer(value) => format!("{value}"),
            Self::Float(value) => value.llvm_syntax(),
            Self::String { value, .. } => format!("{value}"),
            Self::Array { items, .. } => {
                let mut items_iter = items.iter();
                if let Some(item) = items_iter.next() {
                    let mut syntax = String::from("[ ");
                    syntax.push_str(context.type_llvm_syntax(item.get_type()));
                    syntax.push(' ');
                    syntax.push_str(&item.llvm_syntax(context));
                    for item in items_iter {
                        syntax.push_str(", ");
                        syntax.push_str(context.type_llvm_syntax(item.get_type()));
                        syntax.push(' ');
                        syntax.push_str(&item.llvm_syntax(context));
                    }
                    syntax.push_str(" ]");
                    syntax
                }
                else {
                    "[]".to_owned()
                }
            }
            Self::Structure { members, .. } => {
                let mut members_iter = members.iter();
                if let Some(member) = members_iter.next() {
                    let mut syntax = "{ ".to_owned();
                    syntax.push_str(context.type_llvm_syntax(member.get_type()));
                    syntax.push(' ');
                    syntax.push_str(&member.llvm_syntax(context));
                    for member in members_iter {
                        syntax.push_str(", ");
                        syntax.push_str(context.type_llvm_syntax(member.get_type()));
                        syntax.push(' ');
                        syntax.push_str(&member.llvm_syntax(context));
                    }
                    syntax.push_str(" }");
                    syntax
                }
                else {
                    "{}".into()
                }
            }
            Self::Register(register) => register.llvm_syntax(),
            Self::Indirect { pointer, .. } => format!("<ERROR indirect constant: {}>", pointer.llvm_syntax(context)),
            Self::BitwiseCast { value, result_type: to_type } => {
                let value_type = value.get_type();
                let value_syntax = value.llvm_syntax(context);
                format!(
                    "bitcast ({} {value_syntax} to {})",
                    context.type_llvm_syntax(value_type),
                    context.type_llvm_syntax(*to_type),
                )
            }
            Self::GetElementPointer { aggregate_type, pointer, indices, .. } => {
                let mut syntax = format!(
                    "getelementptr inbounds ({}, {} {}",
                    context.type_llvm_syntax(*aggregate_type),
                    context.type_llvm_syntax(pointer.get_type()),
                    pointer.llvm_syntax(context),
                );
                for index in indices {
                    syntax.push_str(", ");
                    syntax.push_str(context.type_llvm_syntax(index.get_type()));
                    syntax.push(' ');
                    syntax.push_str(&index.llvm_syntax(context));
                }
                syntax.push(')');
                syntax
            }
            Self::Type(..) | Self::Module(..) => "<ERROR meta constant>".to_owned(),
        }
    }
}

impl From<bool> for Constant {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<IntegerValue> for Constant {
    fn from(value: IntegerValue) -> Self {
        Self::Integer(value)
    }
}

impl From<FloatValue> for Constant {
    fn from(value: FloatValue) -> Self {
        Self::Float(value)
    }
}

impl From<Register> for Constant {
    fn from(register: Register) -> Self {
        Self::Register(register)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Unresolved(TypeHandle),
    Never,
    Break,
    Continue,
    Void,
    Constant(Constant),
    Register(Register),
    Indirect {
        pointer: Box<Value>,
        pointee_type: TypeHandle,
    },
    BoundFunction {
        self_value: Box<Value>,
        function_value: Box<Value>,
    },
    Type(TypeHandle),
    Module(NamespaceHandle),
}

impl Value {
    pub fn as_namespace(&self, context: &GlobalContext) -> Option<NamespaceHandle> {
        match *self {
            Self::Constant(ref constant) => constant.as_namespace(context),
            Self::Type(handle) => Some(context.type_namespace(handle)),
            Self::Module(namespace) => Some(namespace),
            _ => None
        }
    }

    pub fn as_type(&self) -> Option<TypeHandle> {
        match *self {
            Self::Constant(ref constant) => constant.as_type(),
            Self::Type(handle) => Some(handle),
            _ => None
        }
    }

    pub fn as_module(&self) -> Option<NamespaceHandle> {
        match *self {
            Self::Constant(ref constant) => constant.as_module(),
            Self::Module(namespace) => Some(namespace),
            _ => None
        }
    }

    pub fn get_type(&self) -> TypeHandle {
        match *self {
            Self::Unresolved(handle) => handle,
            Self::Never | Self::Break | Self::Continue => TypeHandle::NEVER,
            Self::Void => TypeHandle::VOID,
            Self::Constant(ref constant) => constant.get_type(),
            Self::Register(ref register) => register.get_type(),
            Self::Indirect { pointee_type, .. } => pointee_type,
            Self::BoundFunction { ref function_value, .. } => function_value.get_type(),
            Self::Type(..) | Self::Module(..) => TypeHandle::META,
        }
    }

    pub fn into_mutable_lvalue(self, context: &GlobalContext) -> crate::Result<(Self, TypeHandle)> {
        match self {
            Self::Indirect { pointer, pointee_type } => {
                if let &TypeRepr::Pointer { semantics: PointerSemantics::Mutable, .. } = context.type_repr(pointer.get_type()) {
                    Ok((*pointer, pointee_type))
                }
                else {
                    Err(Box::new(crate::Error::CannotMutateValue { type_name: pointee_type.path(context).to_string() }))
                }
            }
            _ => {
                Err(Box::new(crate::Error::ExpectedLValue {}))
            }
        }
    }

    pub fn bound_self_value(&self) -> Option<&Value> {
        match self {
            Self::BoundFunction { self_value, .. } => Some(self_value.as_ref()),
            _ => None
        }
    }

    pub fn llvm_syntax(&self, context: &GlobalContext) -> String {
        match self {
            Self::Unresolved(..) => "<ERROR unresolved value>".to_owned(),
            Self::Never | Self::Break | Self::Continue => "<ERROR never value>".to_owned(),
            Self::Void => "<ERROR void value>".to_owned(),
            Self::Constant(constant) => constant.llvm_syntax(context),
            Self::Register(register) => register.llvm_syntax(),
            Self::Indirect { pointer, .. } => format!("<ERROR indirect value: {}>", pointer.llvm_syntax(context)),
            Self::BoundFunction { function_value, .. } => function_value.llvm_syntax(context),
            Self::Type(..) | Self::Module(..) => "<ERROR meta value>".to_owned(),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Constant(Constant::Boolean(value))
    }
}

impl From<IntegerValue> for Value {
    fn from(value: IntegerValue) -> Self {
        Self::Constant(Constant::Integer(value))
    }
}

impl From<FloatValue> for Value {
    fn from(value: FloatValue) -> Self {
        Self::Constant(Constant::Float(value))
    }
}

impl From<Constant> for Value {
    fn from(constant: Constant) -> Self {
        Self::Constant(constant)
    }
}

impl From<Register> for Value {
    fn from(register: Register) -> Self {
        Self::Register(register)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Label {
    identifier: String,
}

impl Label {
    pub fn new(identifier: String) -> Self {
        Self {
            identifier,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn llvm_syntax(&self) -> String {
        format!("%{}", self.identifier())
    }
}
