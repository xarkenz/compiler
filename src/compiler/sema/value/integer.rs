use super::*;

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

    pub fn llvm_syntax(&self) -> String {
        self.to_string()
    }
}

impl std::fmt::Display for IntegerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
