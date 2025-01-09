use super::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PointerSemantics {
    Immutable,
    Mutable,
    Owned,
}

impl PointerSemantics {
    pub fn simple(is_mutable: bool) -> Self {
        if is_mutable {
            Self::Mutable
        }
        else {
            Self::Immutable
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructureMember {
    pub name: String,
    pub member_type: TypeHandle,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FunctionSignature {
    return_type: TypeHandle,
    parameter_types: Box<[TypeHandle]>,
    is_variadic: bool,
}

impl FunctionSignature {
    pub fn new(return_type: TypeHandle, parameter_types: Box<[TypeHandle]>, is_variadic: bool) -> Self {
        Self {
            return_type,
            parameter_types,
            is_variadic,
        }
    }

    pub fn return_type(&self) -> TypeHandle {
        self.return_type
    }

    pub fn parameter_types(&self) -> &[TypeHandle] {
        &self.parameter_types
    }

    pub fn is_variadic(&self) -> bool {
        self.is_variadic
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeInfo {
    Meta,
    Never,
    Void,
    Boolean,
    Integer {
        size: usize,
        signed: bool,
    },
    Pointer {
        pointee_type: TypeHandle,
        semantics: PointerSemantics,
    },
    Array {
        item_type: TypeHandle,
        length: Option<usize>,
    },
    Structure {
        name: String,
        members: Box<[StructureMember]>,
    },
    Function {
        signature: FunctionSignature,
    },
    Undefined {
        name: String,
    },
    Alias {
        target: TypeHandle,
    },
}

/// Order of elements is important! If anything is changed here, `TypeHandle::*` may need to be
/// changed as well.
pub(super) const PRIMITIVE_TYPES: &[(&str, TypeInfo)] = &[
    ("<meta>", TypeInfo::Meta),
    ("never", TypeInfo::Never),
    ("void", TypeInfo::Void),
    ("bool", TypeInfo::Boolean),
    ("i8", TypeInfo::Integer { size: 1, signed: true }),
    ("u8", TypeInfo::Integer { size: 1, signed: false }),
    ("i16", TypeInfo::Integer { size: 2, signed: true }),
    ("u16", TypeInfo::Integer { size: 2, signed: false }),
    ("i32", TypeInfo::Integer { size: 4, signed: true }),
    ("u32", TypeInfo::Integer { size: 4, signed: false }),
    ("i64", TypeInfo::Integer { size: 8, signed: true }),
    ("u64", TypeInfo::Integer { size: 8, signed: false }),
];

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PrimitiveType {
    pub handle: TypeHandle,
    pub name: &'static str,
}

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypeHandle(NonZeroUsize);

impl TypeHandle {
    pub const META: Self = Self::new(0);
    pub const NEVER: Self = Self::new(1);
    pub const VOID: Self = Self::new(2);
    pub const BOOL: Self = Self::new(3);
    pub const I8: Self = Self::new(4);
    pub const U8: Self = Self::new(5);
    pub const I16: Self = Self::new(6);
    pub const U16: Self = Self::new(7);
    pub const I32: Self = Self::new(8);
    pub const U32: Self = Self::new(9);
    pub const I64: Self = Self::new(10);
    pub const U64: Self = Self::new(11);
    
    pub fn primitive(type_name: &str) -> Option<PrimitiveType> {
        PRIMITIVE_TYPES.iter().enumerate().find_map(|(registry_index, &(name, _))| {
            (name == type_name).then_some(PrimitiveType {
                handle: TypeHandle::new(registry_index),
                name,
            })
        })
    }
    
    pub const fn new(registry_index: usize) -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(registry_index + 1) })
    }
    
    pub const fn registry_index(self) -> usize {
        self.0.get() - 1
    }
    
    pub fn info(self, context: &GlobalContext) -> &TypeInfo {
        context.type_info(self)
    }

    pub fn identifier(self, context: &GlobalContext) -> &str {
        context.type_identifier(self)
    }

    pub fn llvm_syntax(self, context: &GlobalContext) -> &str {
        context.type_llvm_syntax(self)
    }
}
