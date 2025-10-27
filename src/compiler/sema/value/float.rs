use super::*;

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

impl std::fmt::Display for FloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float32(value) => write!(f, "{value}"),
            Self::Float64(value) => write!(f, "{value}"),
        }
    }
}
