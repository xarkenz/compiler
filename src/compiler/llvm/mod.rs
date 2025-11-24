use crate::sema::{GlobalContext, TypeRepr};

mod instr;
mod value;
mod types;

pub struct QuotedStringDisplay<T: AsRef<[u8]>>(pub T);

impl<T: AsRef<[u8]>> std::fmt::Display for QuotedStringDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        for &ch in self.0.as_ref() {
            if ch == b'"' || !(ch == b' ' || ch.is_ascii_graphic()) {
                write!(f, "\\{ch:02X}")?;
            }
            else if ch == b'\\' {
                write!(f, "\\\\")?;
            }
            else {
                write!(f, "{}", ch as char)?;
            }
        }
        write!(f, "\"")
    }
}

pub struct IdentifierDisplay<T: AsRef<[u8]>>(pub T);

impl<T: AsRef<[u8]>> std::fmt::Display for IdentifierDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let identifier = self.0.as_ref();
        let needs_quotes = identifier.iter().any(|&ch| {
            !matches!(ch, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b'.' | b'$')
        });

        if needs_quotes {
            QuotedStringDisplay(identifier).fmt(f)
        }
        else {
            // SAFETY: We just validated every byte in the identifier against an ASCII-only list.
            unsafe {
                write!(f, "{}", str::from_utf8_unchecked(identifier))
            }
        }
    }
}

pub trait LLVMDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result;

    fn llvm<'a>(&'a self, context: &'a GlobalContext) -> LLVMDisplayWrapper<'a, Self>
    where
        Self: Sized,
    {
        LLVMDisplayWrapper {
            inner: self,
            context,
        }
    }
}

pub struct LLVMDisplayWrapper<'a, T: LLVMDisplay> {
    pub inner: &'a T,
    pub context: &'a GlobalContext,
}

impl<'a, T: LLVMDisplay> std::fmt::Display for LLVMDisplayWrapper<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f, self.context)
    }
}

impl LLVMDisplay for crate::ir::GlobalVariableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let _ = context;
        match self {
            Self::Constant => write!(f, "constant"),
            Self::AnonymousConstant => write!(f, "private unnamed_addr constant"),
            Self::Mutable => write!(f, "global"),
        }
    }
}

impl LLVMDisplay for crate::ir::ExternalGlobalVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let &TypeRepr::Pointer { pointee_type, .. } = self.register().get_type().repr(context) else {
            panic!("'{}' is not a pointer type", self.register().get_type().path(context));
        };

        write!(
            f,
            "{} = external {} {}",
            self.register().llvm(context),
            self.kind().llvm(context),
            pointee_type.llvm(context),
        )
    }
}

impl LLVMDisplay for crate::ir::ExternalFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let TypeRepr::Function { signature } = self.register().get_type().repr(context) else {
            panic!("'{}' is not a function type", self.register().get_type().path(context));
        };

        write!(
            f,
            "declare {} {}(",
            signature.return_type().llvm(context),
            self.register().llvm(context),
        )?;

        let mut parameters_iter = signature.parameter_types().iter().copied();
        if let Some(parameter_type) = parameters_iter.next() {
            write!(f, "{}", parameter_type.llvm(context))?;
            for parameter_type in parameters_iter {
                write!(f, ", {}", parameter_type.llvm(context))?;
            }
            if signature.is_variadic() {
                write!(f, ", ...")?;
            }
        }
        else if signature.is_variadic() {
            write!(f, "...")?;
        }

        write!(f, ")")
    }
}

impl LLVMDisplay for crate::ir::GlobalVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        write!(
            f,
            "{} = {} {} {}",
            self.register().llvm(context),
            self.kind().llvm(context),
            self.value().get_type().llvm(context),
            self.value().llvm(context),
        )
    }
}

impl LLVMDisplay for crate::ir::FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let TypeRepr::Function { signature } = self.register().get_type().repr(context) else {
            panic!("'{}' is not a function type", self.register().get_type().path(context));
        };

        write!(
            f,
            "define {} {}(",
            signature.return_type().llvm(context),
            self.register().llvm(context),
        )?;

        let mut parameters_iter = self.parameter_registers().iter();
        if let Some(parameter) = parameters_iter.next() {
            write!(f, "{} {}", parameter.get_type().llvm(context), parameter.llvm(context))?;
            for parameter in parameters_iter {
                write!(f, ", {} {}", parameter.get_type().llvm(context), parameter.llvm(context))?;
            }
            if signature.is_variadic() {
                write!(f, ", ...")?;
            }
        }
        else if signature.is_variadic() {
            write!(f, "...")?;
        }

        writeln!(f, ") {{")?;

        for block in self.blocks() {
            block.fmt(f, context)?;
        }

        write!(f, "}}")
    }
}

impl LLVMDisplay for crate::ir::instr::BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        writeln!(f, "{}:", IdentifierDisplay(self.label().identifier()))?;
        for phi in self.phis() {
            writeln!(f, "\t{}", phi.llvm(context))?;
        }
        for instruction in self.body() {
            writeln!(f, "\t{}", instruction.llvm(context))?;
        }
        writeln!(f, "\t{}", self.terminator().llvm(context))
    }
}

impl LLVMDisplay for crate::ir::CompilationUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        let source_filename = self.main_path().as_os_str().as_encoded_bytes();
        writeln!(f, "source_filename = {}", QuotedStringDisplay(&source_filename))?;
        writeln!(f)?;

        for &declared_type in self.type_declarations() {
            types::fmt_type_declaration(f, context, declared_type)?;
            writeln!(f)?;
            writeln!(f)?;
        }
        for variable in self.external_global_variables() {
            writeln!(f, "{}", variable.llvm(context))?;
            writeln!(f)?;
        }
        for function in self.external_functions() {
            writeln!(f, "{}", function.llvm(context))?;
            writeln!(f)?;
        }
        for variable in self.global_variables() {
            writeln!(f, "{}", variable.llvm(context))?;
            writeln!(f)?;
        }
        for function in self.function_definitions() {
            writeln!(f, "{}", function.llvm(context))?;
            writeln!(f)?;
        }

        Ok(())
    }
}
