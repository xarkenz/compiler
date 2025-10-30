use super::*;

mod float;
mod integer;

pub use float::*;
pub use integer::*;

#[derive(Clone, PartialEq, Debug)]
pub struct StringValue {
    bytes: Box<[u8]>,
}

impl StringValue {
    pub fn new(bytes: Box<[u8]>) -> Self {
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

    pub fn llvm_syntax(&self) -> String {
        self.to_string()
    }
}

impl std::fmt::Display for StringValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

fn quote_identifier_if_needed(mut identifier: String) -> Box<str> {
    let needs_quotes = identifier.contains(|ch| {
        !matches!(ch, '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '_' | '.' | '$')
    });

    if needs_quotes {
        identifier.insert(0, '"');
        identifier.push('"');
    }

    identifier.into_boxed_str()
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Register {
    identifier: Box<str>,
    value_type: TypeHandle,
    is_global: bool,
}

impl Register {
    pub fn new_global(raw_identifier: String, value_type: TypeHandle) -> Self {
        Self {
            identifier: quote_identifier_if_needed(raw_identifier),
            value_type,
            is_global: true,
        }
    }

    pub fn new_local(raw_identifier: String, value_type: TypeHandle) -> Self {
        Self {
            identifier: quote_identifier_if_needed(raw_identifier),
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

    pub fn set_type(&mut self, value_type: TypeHandle) {
        self.value_type = value_type;
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

#[derive(Clone, PartialEq, Debug)]
pub struct Label {
    identifier: Box<str>,
}

impl Label {
    pub fn new(raw_identifier: String) -> Self {
        Self {
            identifier: quote_identifier_if_needed(raw_identifier),
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn llvm_syntax(&self) -> String {
        format!("%{}", self.identifier())
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
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
        items: Vec<Self>,
    },
    Tuple {
        tuple_type: TypeHandle,
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
    Convert {
        operation: ConversionOperation,
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
            Self::Integer(ref integer) => integer.integer_type().as_handle(),
            Self::Float(ref float) => float.float_type().as_handle(),
            Self::String { array_type, .. } => array_type,
            Self::Array { array_type, .. } => array_type,
            Self::Tuple { tuple_type, .. } => tuple_type,
            Self::Structure { struct_type, .. } => struct_type,
            Self::Register(ref register) => register.get_type(),
            Self::Indirect { pointee_type, .. } => pointee_type,
            Self::Convert { result_type, .. } => result_type,
            Self::GetElementPointer { result_type, .. } => result_type,
            Self::Type(..) | Self::Module(..) => TypeHandle::META,
        }
    }

    pub fn map_pointer_semantics<F>(&mut self, context: &mut GlobalContext, f: F)
    where
        F: FnOnce(TypeHandle, PointerSemantics) -> PointerSemantics,
    {
        match self {
            Self::Undefined(value_type) => {
                *value_type = value_type.map_pointer_semantics(context, f);
            }
            Self::Poison(value_type) => {
                *value_type = value_type.map_pointer_semantics(context, f);
            }
            Self::ZeroInitializer(value_type) => {
                *value_type = value_type.map_pointer_semantics(context, f);
            }
            Self::NullPointer(value_type) => {
                *value_type = value_type.map_pointer_semantics(context, f);
            }
            Self::Register(register) => {
                register.set_type(register.get_type().map_pointer_semantics(context, f));
            }
            Self::Convert { result_type, .. } => {
                *result_type = result_type.map_pointer_semantics(context, f);
            }
            Self::GetElementPointer { result_type, .. } => {
                *result_type = result_type.map_pointer_semantics(context, f);
            }
            _ => {}
        }
    }

    pub fn llvm_syntax(&self, context: &GlobalContext) -> String {
        match self {
            Self::Undefined(..) => {
                "undef".to_string()
            }
            Self::Poison(..) => {
                "poison".to_string()
            }
            Self::ZeroInitializer(..) => {
                "zeroinitializer".to_string()
            }
            Self::NullPointer(..) => {
                "null".to_string()
            }
            Self::Boolean(value) => {
                value.to_string()
            }
            Self::Integer(value) => {
                value.llvm_syntax()
            }
            Self::Float(value) => {
                value.llvm_syntax()
            }
            Self::String { value, .. } => {
                value.llvm_syntax()
            }
            Self::Array { items, .. } => {
                Self::array_llvm_syntax(items, context)
            }
            Self::Tuple { items, .. } => {
                Self::structure_llvm_syntax(items, context)
            }
            Self::Structure { members, .. } => {
                Self::structure_llvm_syntax(members, context)
            }
            Self::Register(register) => {
                register.llvm_syntax()
            }
            Self::Indirect { pointer, .. } => {
                format!("<ERROR indirect constant: {}>", pointer.llvm_syntax(context))
            }
            Self::Convert { operation, value, result_type: to_type } => {
                let value_type = value.get_type();
                let value_syntax = value.llvm_syntax(context);
                format!(
                    "{operation} ({} {value_syntax} to {})",
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
            Self::Type(..) | Self::Module(..) => {
                "<ERROR meta constant>".to_string()
            }
        }
    }

    fn array_llvm_syntax<'a>(items: impl IntoIterator<Item = &'a Self>, context: &GlobalContext) -> String {
        let mut items = items.into_iter();
        if let Some(item) = items.next() {
            let mut syntax = String::from("[ ");
            syntax.push_str(context.type_llvm_syntax(item.get_type()));
            syntax.push(' ');
            syntax.push_str(&item.llvm_syntax(context));
            for item in items {
                syntax.push_str(", ");
                syntax.push_str(context.type_llvm_syntax(item.get_type()));
                syntax.push(' ');
                syntax.push_str(&item.llvm_syntax(context));
            }
            syntax.push_str(" ]");
            syntax
        }
        else {
            "[]".to_string()
        }
    }

    fn structure_llvm_syntax<'a>(members: impl IntoIterator<Item = &'a Self>, context: &GlobalContext) -> String {
        let mut members = members.into_iter();
        if let Some(member) = members.next() {
            let mut syntax = String::from("{ ");
            syntax.push_str(context.type_llvm_syntax(member.get_type()));
            syntax.push(' ');
            syntax.push_str(&member.llvm_syntax(context));
            for member in members {
                syntax.push_str(", ");
                syntax.push_str(context.type_llvm_syntax(member.get_type()));
                syntax.push(' ');
                syntax.push_str(&member.llvm_syntax(context));
            }
            syntax.push_str(" }");
            syntax
        }
        else {
            "{}".to_string()
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
                if let &TypeRepr::Pointer { semantics: PointerSemantics::Mutable, .. } = pointer.get_type().repr(context) {
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

    pub fn map_pointer_semantics<F>(&mut self, context: &mut GlobalContext, f: F)
    where
        F: FnOnce(TypeHandle, PointerSemantics) -> PointerSemantics,
    {
        match self {
            Self::Constant(constant) => {
                constant.map_pointer_semantics(context, f);
            }
            Self::Register(register) => {
                register.set_type(register.get_type().map_pointer_semantics(context, f));
            }
            _ => {}
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
