use super::*;
use std::num::NonZeroUsize;

mod registry;
pub use registry::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PointerSemantics {
    Immutable,
    Mutable,
    ImmutableSymbol,
}

impl PointerSemantics {
    pub fn normal(is_mutable: bool) -> Self {
        if is_mutable {
            Self::Mutable
        }
        else {
            Self::Immutable
        }
    }

    pub fn for_symbol(is_mutable: bool) -> Self {
        if is_mutable {
            Self::Mutable
        }
        else {
            Self::ImmutableSymbol
        }
    }

    pub fn normalized(self) -> Self {
        match self {
            Self::ImmutableSymbol => Self::Immutable,
            semantics => semantics
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructureMember {
    pub name: Box<str>,
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
pub enum TypeRepr {
    /// A placeholder representation for types that are known to be defined, but where the exact
    /// details are not yet known. This representation is used during the outline phase to
    /// establish a handle for a type before its definition is processed in the fill phase.
    Unresolved,
    /// The representation for the `<meta>` primitive type, which is not directly available to the
    /// programmer, but holds values for modules and types themselves.
    Meta,
    /// The representation for the `never` primitive type, which indicates that control flow will
    /// diverge before the value would be encountered during runtime.
    Never,
    /// The representation for the `void` primitive type, which is effectively "nothing."
    Void,
    /// The representation for the `bool` primitive type, which holds a boolean value.
    Boolean,
    /// The representation for the `iX` and `uX` primitive types, which hold `X`-bit integer values
    /// that are signed and unsigned, respectively. `size` is in *bytes*, not bits.
    Integer {
        size: u64,
        signed: bool,
    },
    /// A placeholder representation for the `isize` and `usize` types which is replaced with the
    /// proper [`TypeRepr::Integer`] representation once pointer size information is known.
    PointerSizedInteger {
        signed: bool,
    },
    /// The representation for the `f32` primitive type, which holds a 32-bit floating-point value.
    Float32,
    /// The representation for the `f64` primitive type, which holds a 64-bit floating-point value.
    Float64,
    /// The representation for pointer types of the form `*T` and `*mut T`.
    Pointer {
        pointee_type: TypeHandle,
        semantics: PointerSemantics,
    },
    /// The representation for array types of the form `[T; N]` and `[T]`.
    Array {
        item_type: TypeHandle,
        length: Option<u64>,
    },
    /// The representation for tuple types.
    Tuple {
        item_types: Box<[TypeHandle]>,
    },
    /// The representation for structure types whose member types are known.
    Structure {
        name: Box<str>,
        members: Box<[StructureMember]>,
    },
    /// The representation for structure types whose layout details are unknown (opaque).
    ForeignStructure {
        name: Box<str>,
    },
    /// The representation for function types, which are defined by their
    /// [signature](FunctionSignature).
    Function {
        signature: FunctionSignature,
    },
}

impl TypeRepr {
    pub fn resolve_primitive_type(&self, pointer_size: u64) -> Self {
        match self {
            &Self::PointerSizedInteger { signed } => Self::Integer {
                size: pointer_size,
                signed,
            },
            other_repr => other_repr.clone(),
        }
    }
}

/// Order of elements is important! If anything is changed here, `TypeHandle::*` may need to be
/// changed as well.
pub(super) const PRIMITIVE_TYPES: &[(&str, TypeRepr)] = &[
    ("<meta>", TypeRepr::Meta),
    ("never", TypeRepr::Never),
    ("void", TypeRepr::Void),
    ("bool", TypeRepr::Boolean),
    ("i8", TypeRepr::Integer { size: 1, signed: true }),
    ("u8", TypeRepr::Integer { size: 1, signed: false }),
    ("i16", TypeRepr::Integer { size: 2, signed: true }),
    ("u16", TypeRepr::Integer { size: 2, signed: false }),
    ("i32", TypeRepr::Integer { size: 4, signed: true }),
    ("u32", TypeRepr::Integer { size: 4, signed: false }),
    ("i64", TypeRepr::Integer { size: 8, signed: true }),
    ("u64", TypeRepr::Integer { size: 8, signed: false }),
    ("isize", TypeRepr::PointerSizedInteger { signed: true }),
    ("usize", TypeRepr::PointerSizedInteger { signed: false }),
    ("f32", TypeRepr::Float32),
    ("f64", TypeRepr::Float64),
];

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PrimitiveType {
    pub handle: TypeHandle,
    pub name: &'static str,
}

impl PrimitiveType {
    pub fn from_name(type_name: &str) -> Option<PrimitiveType> {
        PRIMITIVE_TYPES
            .iter()
            .enumerate()
            .find_map(|(registry_index, &(name, _))| {
                (name == type_name).then(|| PrimitiveType {
                    handle: TypeHandle::new(registry_index),
                    name,
                })
            })
    }
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
    pub const ISIZE: Self = Self::new(12);
    pub const USIZE: Self = Self::new(13);
    pub const F32: Self = Self::new(14);
    pub const F64: Self = Self::new(15);

    pub const fn new(registry_index: usize) -> Self {
        Self(NonZeroUsize::new(registry_index.wrapping_add(1)).unwrap())
    }

    pub const fn registry_index(self) -> usize {
        self.0.get().wrapping_sub(1)
    }

    pub fn repr(self, context: &GlobalContext) -> &TypeRepr {
        context.type_repr(self)
    }

    pub fn path(self, context: &GlobalContext) -> &AbsolutePath {
        context.type_path(self)
    }

    pub fn llvm_syntax(self, context: &GlobalContext) -> &str {
        context.type_llvm_syntax(self)
    }

    pub fn map_pointer_semantics<F>(self, context: &mut GlobalContext, f: F) -> Self
    where
        F: FnOnce(TypeHandle, PointerSemantics) -> PointerSemantics,
    {
        if let &TypeRepr::Pointer { pointee_type, semantics } = self.repr(context) {
            let semantics = f(pointee_type, semantics);
            context.get_pointer_type(pointee_type, semantics)
        }
        else {
            self
        }
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

#[derive(Copy, Clone, Debug)]
pub struct Conversion {
    pub operation_needed: Option<ConversionOperation>,
    pub implicit_allowed: bool,
}

impl Conversion {
    pub fn try_implicit(registry: &TypeRegistry, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> Option<Self> {
        if from_type == TypeHandle::NEVER || from_type == to_type {
            return Some(Self {
                operation_needed: None,
                implicit_allowed: true,
            });
        }

        match (registry.type_repr(from_type), registry.type_repr(to_type)) {
            (
                &TypeRepr::Pointer { pointee_type: from_pointee, semantics: from_semantics },
                &TypeRepr::Pointer { pointee_type: to_pointee, semantics: to_semantics },
            ) => {
                use PointerSemantics::*;
                match (from_semantics, to_semantics) {
                    (Immutable | ImmutableSymbol, Immutable | ImmutableSymbol) => {
                        Self::try_implicit(registry, from_pointee, to_pointee, false)
                    }
                    (Mutable, Immutable | ImmutableSymbol) => {
                        Self::try_implicit(registry, from_pointee, to_pointee, true)
                    }
                    (Mutable, Mutable) if from_mutable => {
                        Self::try_implicit(registry, from_pointee, to_pointee, true)
                    }
                    _ => None
                }
            }
            (
                &TypeRepr::Array { item_type: from_item, length: Some(from_length) },
                &TypeRepr::Array { item_type: to_item, length: Some(to_length) },
            ) => {
                if from_length != to_length {
                    return None;
                }
                Self::try_implicit(registry, from_item, to_item, from_mutable)
            }
            (
                &TypeRepr::Array { item_type: from_item, length: from_length },
                &TypeRepr::Array { item_type: to_item, length: None },
            ) => {
                Self::try_implicit(registry, from_item, to_item, from_mutable)
                    .map(|mut conversion| {
                        if from_length.is_some() {
                            conversion.operation_needed = Some(ConversionOperation::BitwiseCast);
                        }
                        conversion
                    })
            }
            _ => None
        }
    }

    pub fn try_explicit(registry: &TypeRegistry, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> Option<Self> {
        if let Some(conversion) = Self::try_implicit(registry, from_type, to_type, from_mutable) {
            return Some(conversion);
        }

        let operation_needed = match (registry.type_repr(from_type), registry.type_repr(to_type)) {
            (
                &TypeRepr::Integer { size: from_size, signed: from_signed },
                &TypeRepr::Integer { size: to_size, .. },
            ) => {
                use std::cmp::Ordering::*;
                match from_size.cmp(&to_size) {
                    Greater => Some(ConversionOperation::Truncate),
                    Less if from_signed => Some(ConversionOperation::SignExtend),
                    Less => Some(ConversionOperation::ZeroExtend),
                    Equal => None,
                }
            }
            (TypeRepr::Boolean, TypeRepr::Integer { .. }) => {
                Some(ConversionOperation::ZeroExtend)
            }
            (TypeRepr::Float64, TypeRepr::Float32) => {
                Some(ConversionOperation::FloatTruncate)
            }
            (TypeRepr::Float32, TypeRepr::Float64) => {
                Some(ConversionOperation::FloatExtend)
            }
            (
                TypeRepr::Float32 | TypeRepr::Float64,
                &TypeRepr::Integer { signed, .. },
            ) => {
                match signed {
                    true => Some(ConversionOperation::SignedToFloat),
                    false => Some(ConversionOperation::UnsignedToFloat),
                }
            }
            (
                &TypeRepr::Integer { signed, .. },
                TypeRepr::Float32 | TypeRepr::Float64,
            ) => {
                match signed {
                    true => Some(ConversionOperation::FloatToSigned),
                    false => Some(ConversionOperation::FloatToUnsigned),
                }
            }
            (TypeRepr::Boolean, TypeRepr::Float32 | TypeRepr::Float64) => {
                Some(ConversionOperation::UnsignedToFloat)
            }
            (
                TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
                TypeRepr::Integer { .. },
            ) => {
                Some(ConversionOperation::PointerToInteger)
            }
            (
                TypeRepr::Integer { .. },
                TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
            ) => {
                Some(ConversionOperation::IntegerToPointer)
            }
            (
                TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
                TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
            ) => {
                Some(ConversionOperation::BitwiseCast)
            }
            _ => return None
        };

        Some(Self {
            operation_needed,
            implicit_allowed: false,
        })
    }
}
