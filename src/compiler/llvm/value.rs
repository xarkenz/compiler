use super::*;

impl LLVMDisplay for crate::ir::value::IntegerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let _ = context;
        write!(f, "{self}")
    }
}

impl LLVMDisplay for crate::ir::value::FloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let _ = context;
        // Convert to hexadecimal representation for purposes of keeping exact value
        write!(f, "0x{:016X}", self.raw().to_bits())
    }
}

impl LLVMDisplay for crate::ir::value::StringValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let _ = context;
        write!(f, "c{}", QuotedStringDisplay(self.bytes()))
    }
}

impl LLVMDisplay for crate::ir::value::LocalRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let _ = context;
        write!(f, "%{}", IdentifierDisplay(self.identifier()))
    }
}

impl LLVMDisplay for crate::ir::value::GlobalRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let _ = context;
        write!(f, "@{}", IdentifierDisplay(self.identifier()))
    }
}

impl LLVMDisplay for crate::ir::value::BlockLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let _ = context;
        write!(f, "%{}", IdentifierDisplay(self.identifier()))
    }
}

impl LLVMDisplay for crate::ir::value::Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        fn fmt_aggregate<'a>(
            f: &mut std::fmt::Formatter<'_>,
            context: &GlobalContext,
            items: impl IntoIterator<Item = &'a crate::ir::value::Constant>,
            left_bracket: u8,
        ) -> std::fmt::Result {
            let left = left_bracket as char;
            // We can calculate the right bracket because the only possible values of left_bracket
            // are '[' and '{', which both have the right variant 2 values later.
            let right = left_bracket.wrapping_add(2) as char;
            let mut items = items.into_iter();
            if let Some(item) = items.next() {
                write!(f, "{left} {} {}", item.get_type().llvm(context), item.llvm(context))?;
                for item in items {
                    write!(f, ", {} {}", item.get_type().llvm(context), item.llvm(context))?;
                }
                write!(f, " {right}")
            }
            else {
                write!(f, "{left}{right}")
            }
        }

        match self {
            Self::Undefined(..) => {
                write!(f, "undef")
            }
            Self::Poison(..) => {
                write!(f, "poison")
            }
            Self::ZeroInitializer(..) => {
                write!(f, "zeroinitializer")
            }
            Self::NullPointer(..) => {
                write!(f, "null")
            }
            Self::Boolean(value) => {
                write!(f, "{value}")
            }
            Self::Integer(value) => {
                value.fmt(f, context)
            }
            Self::Float(value) => {
                value.fmt(f, context)
            }
            Self::String { value, .. } => {
                value.fmt(f, context)
            }
            Self::Array { items, .. } => {
                fmt_aggregate(f, context, items, b'[')
            }
            Self::Tuple { items, .. } => {
                fmt_aggregate(f, context, items, b'{')
            }
            Self::Structure { members, .. } => {
                fmt_aggregate(f, context, members, b'{')
            }
            Self::Register(register) => {
                register.fmt(f, context)
            }
            Self::Indirect { pointer, .. } => {
                write!(f, "<ERROR indirect constant: {}>", pointer.llvm(context))
            }
            Self::Convert { operation, value, result_type } => {
                write!(
                    f,
                    "{} ({} {} to {})",
                    operation.llvm(context),
                    value.get_type().llvm(context),
                    value.llvm(context),
                    result_type.llvm(context),
                )
            }
            Self::GetElementPointer { aggregate_type, pointer, indices, .. } => {
                write!(
                    f,
                    "getelementptr inbounds ({}, {} {}",
                    aggregate_type.llvm(context),
                    pointer.get_type().llvm(context),
                    pointer.llvm(context),
                )?;
                for index in indices {
                    write!(f, ", {} {}", index.get_type().llvm(context), index.llvm(context))?;
                }
                write!(f, ")")
            }
            Self::Type(..) | Self::Module(..) => {
                write!(f, "<ERROR meta constant>")
            }
        }
    }
}

impl LLVMDisplay for crate::ir::value::Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        match self {
            Self::Never | Self::Break | Self::Continue => {
                write!(f, "<ERROR never value>")
            }
            Self::Void => {
                write!(f, "<ERROR void value>")
            }
            Self::Constant(constant) => {
                constant.fmt(f, context)
            }
            Self::Register(register) => {
                register.fmt(f, context)
            }
            Self::Indirect { pointer, .. } => {
                write!(f, "<ERROR indirect value: {}>", pointer.llvm(context))
            }
            Self::BoundFunction { function_value, .. } => {
                function_value.fmt(f, context)
            }
        }
    }
}