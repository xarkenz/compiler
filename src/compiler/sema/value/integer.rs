use super::*;

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

    pub fn llvm_syntax(&self) -> String {
        self.to_string()
    }
}

impl std::fmt::Display for IntegerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}
