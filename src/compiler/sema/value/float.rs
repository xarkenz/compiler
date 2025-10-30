use super::*;

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

    pub fn llvm_syntax(&self) -> String {
        // Convert to hexadecimal representation for purposes of keeping exact value
        format!("0x{:016X}", self.raw.to_bits())
    }
}

impl std::fmt::Display for FloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}
