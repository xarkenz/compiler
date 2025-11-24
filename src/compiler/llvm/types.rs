use super::*;
use crate::sema::{StructureMember, TypeHandle};

impl LLVMDisplay for TypeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        match self.repr(context) {
            TypeRepr::Unresolved => {
                write!(f, "<ERROR unresolved type>")
            }
            TypeRepr::Meta => {
                write!(f, "<ERROR meta type>")
            }
            TypeRepr::Never | TypeRepr::Void => {
                write!(f, "void")
            }
            TypeRepr::Boolean => {
                write!(f, "i1")
            }
            TypeRepr::Integer { size, .. } => {
                write!(f, "i{}", size * 8)
            }
            TypeRepr::PointerSizedInteger { .. } => {
                panic!("unresolved pointer sized integer")
            }
            TypeRepr::Float32 => {
                write!(f, "float")
            }
            TypeRepr::Float64 => {
                write!(f, "double")
            }
            TypeRepr::Pointer { pointee_type, .. } => match *pointee_type {
                TypeHandle::NEVER | TypeHandle::VOID => {
                    write!(f, "{{}}*")
                }
                pointee_type => {
                    write!(f, "{}*", pointee_type.llvm(context))
                }
            }
            TypeRepr::Array { item_type, length } => match length {
                Some(length) => {
                    write!(f, "[{} x {}]", length, item_type.llvm(context))
                }
                None => {
                    item_type.fmt(f, context)
                }
            }
            TypeRepr::Tuple { ref item_types } => match *item_types.as_ref() {
                [] => {
                    write!(f, "{{}}")
                }
                [first_item_type, ref item_types @ ..] => {
                    write!(f, "{{ {}", first_item_type.llvm(context))?;
                    for &item_type in item_types {
                        write!(f, ", {}", item_type.llvm(context))?;
                    }
                    write!(f, " }}")
                }
            }
            TypeRepr::Structure { .. } | TypeRepr::OpaqueStructure { .. } => {
                // We could use IdentifierDisplay here, but it will always end up quoting the path,
                // and none of the characters will need to be escaped. This is simpler.
                write!(f, "%\"{}\"", self.path(context))
            }
            TypeRepr::Function { ref signature } => {
                write!(f, "{}(", signature.return_type().llvm(context))?;
                let mut parameters_iter = signature.parameter_types().iter();
                if let Some(&parameter) = parameters_iter.next() {
                    write!(f, "{}", parameter.llvm(context))?;
                    for &parameter in parameters_iter {
                        write!(f, ", {}", parameter.llvm(context))?;
                    }
                    if signature.is_variadic() {
                        write!(f, ", ...")?;
                    }
                }
                else if signature.is_variadic() {
                    write!(f, "...")?;
                }
                write!(f, ")*")
            }
        }
    }
}

impl LLVMDisplay for crate::sema::ConversionOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter, context: &GlobalContext) -> std::fmt::Result {
        let _ = context;
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

pub fn fmt_type_declaration(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    type_handle: TypeHandle,
) -> std::fmt::Result {
    let type_llvm = type_handle.llvm(context);

    match type_handle.repr(context) {
        TypeRepr::Structure { members, .. } => {
            write!(f, "{type_llvm} = type ")?;
            let mut members_iter = members.iter();
            if let Some(&StructureMember { member_type, .. }) = members_iter.next() {
                write!(f, "{{ {}", member_type.llvm(context))?;
                for &StructureMember { member_type, .. } in members_iter {
                    write!(f, ", {}", member_type.llvm(context))?;
                }
                write!(f, " }}")
            }
            else {
                write!(f, "{{}}")
            }
        }
        TypeRepr::OpaqueStructure { .. } => {
            write!(f, "{type_llvm} = type opaque")
        }
        _ => {
            // The type does not need to be declared since its definition is inherent.
            Ok(())
        }
    }
}
