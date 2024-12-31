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
pub(super) const PRIMITIVE_TYPES: &[(TypeInfo, &str)] = &[
    (TypeInfo::Never, "never"),
    (TypeInfo::Void, "void"),
    (TypeInfo::Boolean, "bool"),
    (TypeInfo::Integer { size: 1, signed: true }, "i8"),
    (TypeInfo::Integer { size: 1, signed: false }, "u8"),
    (TypeInfo::Integer { size: 2, signed: true }, "i16"),
    (TypeInfo::Integer { size: 2, signed: false }, "u16"),
    (TypeInfo::Integer { size: 4, signed: true }, "i32"),
    (TypeInfo::Integer { size: 4, signed: false }, "u32"),
    (TypeInfo::Integer { size: 8, signed: true }, "i64"),
    (TypeInfo::Integer { size: 8, signed: false }, "u64"),
];

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypeHandle(NonZeroUsize);

impl TypeHandle {
    pub const NEVER: TypeHandle = TypeHandle::new(0);
    pub const VOID: TypeHandle = TypeHandle::new(1);
    pub const BOOL: TypeHandle = TypeHandle::new(2);
    pub const I8: TypeHandle = TypeHandle::new(3);
    pub const U8: TypeHandle = TypeHandle::new(4);
    pub const I16: TypeHandle = TypeHandle::new(5);
    pub const U16: TypeHandle = TypeHandle::new(6);
    pub const I32: TypeHandle = TypeHandle::new(7);
    pub const U32: TypeHandle = TypeHandle::new(8);
    pub const I64: TypeHandle = TypeHandle::new(9);
    pub const U64: TypeHandle = TypeHandle::new(10);
    
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
