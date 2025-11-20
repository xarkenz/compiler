use super::*;

mod float;
mod integer;

pub use float::*;
pub use integer::*;
use crate::gen::llvm::EscapedStringDisplay;

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
        write!(f, "c{}", EscapedStringDisplay(self.bytes()))
    }
}

fn quote_identifier_if_needed(mut identifier: String) -> Box<str> {
    let needs_quotes = identifier.contains(|ch| {
        !matches!(ch, '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '_' | '.' | '$')
    });

    if needs_quotes {
        identifier = EscapedStringDisplay(&identifier).to_string();
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

    pub fn as_namespace(&self, context: &GlobalContext) -> Option<NamespaceHandle> {
        match *self {
            Self::Type(handle) => Some(context.type_namespace(handle)),
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

    pub fn set_type(&mut self, handle: TypeHandle) {
        match self {
            Self::Undefined(value_type) => *value_type = handle,
            Self::Poison(value_type) => *value_type = handle,
            Self::ZeroInitializer(value_type) => *value_type = handle,
            Self::NullPointer(value_type) => *value_type = handle,
            Self::Integer(integer) => {
                integer.set_integer_type(IntegerType::from_handle(handle)
                    .expect("failed to set integer type"));
            }
            Self::Float(float) => {
                float.set_float_type(FloatType::from_handle(handle)
                    .expect("failed to set integer type"));
            }
            Self::String { array_type, .. } => *array_type = handle,
            Self::Array { array_type, .. } => *array_type = handle,
            Self::Tuple { tuple_type, .. } => *tuple_type = handle,
            Self::Structure { struct_type, .. } => *struct_type = handle,
            Self::Register(register) => register.set_type(handle),
            Self::Convert { result_type, .. } => *result_type = handle,
            Self::GetElementPointer { result_type, .. } => *result_type = handle,
            _ => {}
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
            Self::Convert { operation, value, result_type } => {
                let value_type = value.get_type();
                let value_syntax = value.llvm_syntax(context);
                format!(
                    "{operation} ({} {value_syntax} to {})",
                    context.type_llvm_syntax(value_type),
                    context.type_llvm_syntax(*result_type),
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
        pointee_type: TypeHandle,
    },
    BoundFunction {
        self_value: Box<(crate::Span, Value)>,
        function_value: Box<Value>,
    },
}

impl Value {
    pub fn as_constant(&self) -> Option<&Constant> {
        match self {
            Self::Constant(constant) => Some(constant),
            _ => None
        }
    }

    pub fn as_type(&self) -> Option<TypeHandle> {
        self.as_constant()?.as_type()
    }

    pub fn as_module(&self) -> Option<NamespaceHandle> {
        self.as_constant()?.as_module()
    }

    pub fn as_namespace(&self, context: &GlobalContext) -> Option<NamespaceHandle> {
        self.as_constant()?.as_namespace(context)
    }

    pub fn get_type(&self) -> TypeHandle {
        match *self {
            Self::Never | Self::Break | Self::Continue => TypeHandle::NEVER,
            Self::Void => TypeHandle::VOID,
            Self::Constant(ref constant) => constant.get_type(),
            Self::Register(ref register) => register.get_type(),
            Self::Indirect { pointee_type, .. } => pointee_type,
            Self::BoundFunction { ref function_value, .. } => function_value.get_type(),
        }
    }

    pub fn set_type(&mut self, handle: TypeHandle) {
        match self {
            Self::Constant(constant) => constant.set_type(handle),
            Self::Register(register) => register.set_type(handle),
            _ => {}
        }
    }

    pub fn into_mutable_lvalue(self, span: crate::Span, context: &GlobalContext) -> crate::Result<(Self, TypeHandle)> {
        match self {
            Self::Indirect { pointer, pointee_type } => {
                if let &TypeRepr::Pointer { semantics: PointerSemantics::Mutable, .. } = pointer.get_type().repr(context) {
                    Ok((*pointer, pointee_type))
                }
                else {
                    Err(Box::new(crate::Error::new(
                        Some(span),
                        crate::ErrorKind::CannotMutateValue {
                            type_name: pointee_type.path(context).to_string(),
                        },
                    )))
                }
            }
            _ => {
                Err(Box::new(crate::Error::new(
                    Some(span),
                    crate::ErrorKind::ExpectedLValue,
                )))
            }
        }
    }

    pub fn bound_self_value(&self) -> Option<&Value> {
        match self {
            Self::BoundFunction { self_value, .. } => Some(&self_value.1),
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
            Self::Never | Self::Break | Self::Continue => {
                "<ERROR never value>".to_string()
            }
            Self::Void => {
                "<ERROR void value>".to_string()
            }
            Self::Constant(constant) => {
                constant.llvm_syntax(context)
            }
            Self::Register(register) => {
                register.llvm_syntax()
            }
            Self::Indirect { pointer, .. } => {
                format!("<ERROR indirect value: {}>", pointer.llvm_syntax(context))
            }
            Self::BoundFunction { function_value, .. } => {
                function_value.llvm_syntax(context)
            }
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
