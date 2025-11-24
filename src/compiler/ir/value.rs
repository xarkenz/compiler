use crate::sema::*;
use crate::target::TargetInfo;

#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum IntegerType {
    I8 = TypeHandle::I8.registry_index(),
    U8 = TypeHandle::U8.registry_index(),
    I16 = TypeHandle::I16.registry_index(),
    U16 = TypeHandle::U16.registry_index(),
    I32 = TypeHandle::I32.registry_index(),
    U32 = TypeHandle::U32.registry_index(),
    I64 = TypeHandle::I64.registry_index(),
    U64 = TypeHandle::U64.registry_index(),
    Isize = TypeHandle::ISIZE.registry_index(),
    Usize = TypeHandle::USIZE.registry_index(),
}

impl IntegerType {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "i8" => Some(Self::I8),
            "u8" => Some(Self::U8),
            "i16" => Some(Self::I16),
            "u16" => Some(Self::U16),
            "i32" => Some(Self::I32),
            "u32" => Some(Self::U32),
            "i64" => Some(Self::I64),
            "u64" => Some(Self::U64),
            "isize" => Some(Self::Isize),
            "usize" => Some(Self::Usize),
            _ => None
        }
    }

    pub fn from_handle(handle: TypeHandle) -> Option<Self> {
        match handle {
            TypeHandle::I8 => Some(Self::I8),
            TypeHandle::U8 => Some(Self::U8),
            TypeHandle::I16 => Some(Self::I16),
            TypeHandle::U16 => Some(Self::U16),
            TypeHandle::I32 => Some(Self::I32),
            TypeHandle::U32 => Some(Self::U32),
            TypeHandle::I64 => Some(Self::I64),
            TypeHandle::U64 => Some(Self::U64),
            TypeHandle::ISIZE => Some(Self::Isize),
            TypeHandle::USIZE => Some(Self::Usize),
            _ => None
        }
    }

    pub fn as_handle(&self) -> TypeHandle {
        TypeHandle::new(*self as usize)
    }

    pub fn size(&self, target: &TargetInfo) -> u64 {
        match self {
            Self::I8 | Self::U8 => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 => 4,
            Self::I64 | Self::U64 => 8,
            Self::Isize | Self::Usize => target.pointer_size(),
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::Isize => true,
            Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::Usize => false,
        }
    }
}

impl std::fmt::Display for IntegerType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::I8 => write!(f, "i8"),
            Self::U8 => write!(f, "u8"),
            Self::I16 => write!(f, "i16"),
            Self::U16 => write!(f, "u16"),
            Self::I32 => write!(f, "i32"),
            Self::U32 => write!(f, "u32"),
            Self::I64 => write!(f, "i64"),
            Self::U64 => write!(f, "u64"),
            Self::Isize => write!(f, "isize"),
            Self::Usize => write!(f, "usize"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct IntegerValue {
    integer_type: IntegerType,
    raw: i128,
}

impl IntegerValue {
    pub fn new(integer_type: IntegerType, raw: i128) -> Self {
        Self {
            integer_type,
            raw,
        }
    }

    pub fn from_unknown_type(raw: i128, type_handle: TypeHandle, target: &TargetInfo) -> Option<Self> {
        let integer_type = IntegerType::from_handle(type_handle)?;
        let size = integer_type.size(target);
        let signed = integer_type.is_signed();

        let raw = match (size, signed) {
            (1, true) => raw as i8 as i128,
            (1, false) => raw as u8 as i128,
            (2, true) => raw as i16 as i128,
            (2, false) => raw as u16 as i128,
            (4, true) => raw as i32 as i128,
            (4, false) => raw as u32 as i128,
            (8, true) => raw as i64 as i128,
            (8, false) => raw as u64 as i128,
            _ => return None
        };

        Some(Self {
            integer_type,
            raw,
        })
    }

    pub fn raw(&self) -> i128 {
        self.raw
    }

    pub fn integer_type(&self) -> IntegerType {
        self.integer_type
    }

    pub fn set_integer_type(&mut self, integer_type: IntegerType) {
        self.integer_type = integer_type;
    }
}

impl std::fmt::Display for IntegerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FloatType {
    F32 = TypeHandle::F32.registry_index(),
    F64 = TypeHandle::F64.registry_index(),
}

impl FloatType {
    pub fn from_name(syntax: &str) -> Option<Self> {
        match syntax {
            "f32" => Some(Self::F32),
            "f64" => Some(Self::F64),
            _ => None
        }
    }

    pub fn from_handle(handle: TypeHandle) -> Option<Self> {
        match handle {
            TypeHandle::F32 => Some(Self::F32),
            TypeHandle::F64 => Some(Self::F64),
            _ => None
        }
    }

    pub fn as_handle(&self) -> TypeHandle {
        TypeHandle::new(*self as usize)
    }

    pub fn size(&self, target: &TargetInfo) -> u64 {
        let _ = target;
        match self {
            Self::F32 => 4,
            Self::F64 => 8,
        }
    }
}

impl std::fmt::Display for FloatType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct FloatValue {
    float_type: FloatType,
    raw: f64,
}

impl FloatValue {
    pub fn new(float_type: FloatType, raw: f64) -> Self {
        Self {
            float_type,
            raw,
        }
    }

    pub fn from_unknown_type(raw: f64, type_handle: TypeHandle, target: &TargetInfo) -> Option<Self> {
        let float_type = FloatType::from_handle(type_handle)?;
        let size = float_type.size(target);

        let raw = match size {
            4 => raw as f32 as f64,
            8 => raw,
            _ => return None
        };

        Some(Self {
            float_type,
            raw,
        })
    }

    pub fn raw(&self) -> f64 {
        self.raw
    }

    pub fn float_type(&self) -> FloatType {
        self.float_type
    }

    pub fn set_float_type(&mut self, float_type: FloatType) {
        self.float_type = float_type;
    }
}

impl std::fmt::Display for FloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

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
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LocalRegister {
    identifier: Box<str>,
    value_type: TypeHandle,
}

impl LocalRegister {
    pub fn new(identifier: Box<str>, value_type: TypeHandle) -> Self {
        Self {
            identifier,
            value_type,
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
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GlobalRegister {
    identifier: Box<str>,
    value_type: TypeHandle,
}

impl GlobalRegister {
    pub fn new(identifier: Box<str>, value_type: TypeHandle) -> Self {
        Self {
            identifier,
            value_type,
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
}

#[derive(Clone, PartialEq, Debug)]
pub struct BlockLabel {
    identifier: Box<str>,
}

impl BlockLabel {
    pub fn new(identifier: Box<str>) -> Self {
        Self {
            identifier,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
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
    Register(GlobalRegister),
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

impl From<GlobalRegister> for Constant {
    fn from(register: GlobalRegister) -> Self {
        Self::Register(register)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Never,
    Break,
    Continue,
    Void,
    Constant(Constant),
    Register(LocalRegister),
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

impl From<GlobalRegister> for Value {
    fn from(register: GlobalRegister) -> Self {
        Self::Constant(Constant::Register(register))
    }
}

impl From<Constant> for Value {
    fn from(constant: Constant) -> Self {
        Self::Constant(constant)
    }
}

impl From<LocalRegister> for Value {
    fn from(register: LocalRegister) -> Self {
        Self::Register(register)
    }
}
