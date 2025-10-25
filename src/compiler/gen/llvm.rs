use crate::sema::*;

use std::io::Write;

macro_rules! emit {
    ($emitter:expr, $($args:tt)*) => {
        write!(($emitter).writer, $($args)*).map_err(|cause| ($emitter).error(cause))
    };
}

pub struct Emitter<W: Write> {
    filename: String,
    writer: W,
    is_global: bool,
    queued_global_declarations: Vec<String>,
}

impl Emitter<std::fs::File> {
    pub fn from_filename(filename: String) -> crate::Result<Self> {
        std::fs::File::create(&filename)
            .map(|file| Self::new(filename.clone(), file))
            .map_err(|cause| Box::new(crate::Error::OutputFileWrite { filename: filename.clone(), cause }))
    }
}

impl<W: Write> Emitter<W> {
    pub fn new(filename: String, writer: W) -> Self {
        Self {
            filename,
            writer,
            is_global: true,
            queued_global_declarations: Vec::new(),
        }
    }

    pub fn filename(&self) -> &str {
        self.filename.as_str()
    }

    fn error(&self, cause: std::io::Error) -> Box<crate::Error> {
        Box::new(crate::Error::OutputFileWrite { filename: self.filename.clone(), cause })
    }

    pub fn emit_preamble(&mut self, file_id: usize, source_filename: &str) -> crate::Result<()> {
        emit!(self, "; file_id = {file_id}\nsource_filename = \"{source_filename}\"\n\n")
    }

    pub fn emit_postamble(&mut self) -> crate::Result<()> {
        // TODO: metadata
        Ok(())
    }

    pub fn emit_function_declaration(&mut self, function: &Register, context: &GlobalContext) -> crate::Result<()> {
        let TypeRepr::Function { signature } = function.get_type().repr(context) else {
            panic!("{} is not a valid function", function.llvm_syntax());
        };

        emit!(self, "declare {} {}(", signature.return_type().llvm_syntax(context), function.llvm_syntax())?;

        let mut parameters_iter = signature.parameter_types().iter().copied();
        if let Some(parameter_type) = parameters_iter.next() {
            emit!(self, "{}", parameter_type.llvm_syntax(context))?;
            for parameter_type in parameters_iter {
                emit!(self, ", {}", parameter_type.llvm_syntax(context))?;
            }
            if signature.is_variadic() {
                emit!(self, ", ...")?;
            }
        }
        else if signature.is_variadic() {
            emit!(self, "...")?;
        }

        emit!(self, ")\n\n")
    }

    pub fn emit_function_enter(&mut self, function: &Register, parameters: &[Register], context: &GlobalContext) -> crate::Result<()> {
        let TypeRepr::Function { signature } = function.get_type().repr(context) else {
            panic!("{} is not a valid function", function.llvm_syntax());
        };

        emit!(self, "define {} {}(", signature.return_type().llvm_syntax(context), function.llvm_syntax())?;

        let mut parameters_iter = parameters.iter();
        if let Some(parameter) = parameters_iter.next() {
            emit!(self, "{} {}", parameter.get_type().llvm_syntax(context), parameter.llvm_syntax())?;
            for parameter in parameters_iter {
                emit!(self, ", {} {}", parameter.get_type().llvm_syntax(context), parameter.llvm_syntax())?;
            }
            if signature.is_variadic() {
                emit!(self, ", ...")?;
            }
        }
        else if signature.is_variadic() {
            emit!(self, "...")?;
        }

        if !self.is_global {
            panic!("entering a function while already within a function");
        }
        self.is_global = false;

        emit!(self, ") {{\n")
    }

    pub fn emit_function_exit(&mut self) -> crate::Result<()> {
        emit!(self, "}}\n\n")?;

        self.is_global = true;

        // Write all constant declarations queued for writing
        for constant in &self.queued_global_declarations {
            emit!(self, "{constant}\n")?;
        }
        if !self.queued_global_declarations.is_empty() {
            emit!(self, "\n")?;
        }
        // Dequeue all declarations which were just written
        self.queued_global_declarations.clear();

        Ok(())
    }

    pub fn emit_type_declaration(&mut self, type_handle: TypeHandle, context: &GlobalContext) -> crate::Result<()> {
        let syntax = type_handle.llvm_syntax(context);

        emit!(self, "{syntax} = type opaque\n\n")
    }

    pub fn emit_type_definition(&mut self, type_handle: TypeHandle, context: &GlobalContext) -> crate::Result<()> {
        let syntax = type_handle.llvm_syntax(context);

        let TypeRepr::Structure { members, .. } = type_handle.repr(context) else {
            panic!("{} is not a structure type", type_handle.path(context));
        };

        emit!(self, "{syntax} = type ")?;
        let mut members_iter = members.iter();
        if let Some(&StructureMember { member_type, .. }) = members_iter.next() {
            emit!(self, "{{ {}", member_type.llvm_syntax(context))?;
            for &StructureMember { member_type, .. } in members_iter {
                emit!(self, ", {}", member_type.llvm_syntax(context))?;
            }
            emit!(self, " }}\n\n")
        }
        else {
            emit!(self, "{{}}\n\n")
        }
    }

    pub fn emit_global_allocation(&mut self, pointer: &Register, value: &Constant, is_constant: bool, context: &GlobalContext) -> crate::Result<()> {
        let declaration = format!(
            "{} = {} {} {}",
            pointer.llvm_syntax(),
            if is_constant { "constant" } else { "global" },
            value.get_type().llvm_syntax(context),
            value.llvm_syntax(context),
        );
        if self.is_global {
            // Write the constant declaration immediately
            emit!(self, "{declaration}\n\n")
        }
        else {
            // Enqueue the constant declaration so it will be written just after the function end
            self.queued_global_declarations.push(declaration);
            Ok(())
        }
    }

    pub fn emit_anonymous_constant(&mut self, pointer: &Register, value: &Constant, context: &GlobalContext) -> crate::Result<()> {
        let declaration = format!(
            "{} = private unnamed_addr constant {} {}",
            pointer.llvm_syntax(),
            value.get_type().llvm_syntax(context),
            value.llvm_syntax(context),
        );
        if self.is_global {
            // Write the constant declaration immediately
            emit!(self, "{declaration}\n")
        }
        else {
            // Enqueue the constant declaration so it will be written just after the function end
            self.queued_global_declarations.push(declaration);
            Ok(())
        }
    }

    pub fn emit_label(&mut self, label: &Label) -> crate::Result<()> {
        emit!(self, "{}:\n", label.identifier())
    }

    pub fn emit_unconditional_branch(&mut self, label: &Label) -> crate::Result<()> {
        emit!(self, "\tbr label {}\n", label.llvm_syntax())
    }

    pub fn emit_conditional_branch(&mut self, condition: &Value, consequent: &Label, alternative: &Label, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\tbr i1 {}, label {}, label {}\n",
            condition.llvm_syntax(context),
            consequent.llvm_syntax(),
            alternative.llvm_syntax(),
        )
    }

    pub fn emit_phi<'a, I>(&mut self, result: &Register, inputs: I, context: &GlobalContext) -> crate::Result<()>
    where
        I: IntoIterator<Item = (&'a Value, &'a Label)>,
    {
        let mut inputs = inputs.into_iter();

        let (value, label) = inputs.next()
            .expect("inputs cannot be empty");
        emit!(
            self,
            "\t{} = phi {} [ {}, {} ]",
            result.llvm_syntax(),
            result.get_type().llvm_syntax(context),
            value.llvm_syntax(context),
            label.llvm_syntax(),
        )?;

        for (value, label) in inputs {
            emit!(self, ", [ {}, {} ]", value.llvm_syntax(context), label.llvm_syntax())?;
        }

        emit!(self, "\n")
    }

    pub fn emit_local_allocation(&mut self, pointer: &Register, context: &GlobalContext) -> crate::Result<()> {
        let &TypeRepr::Pointer { pointee_type, .. } = pointer.get_type().repr(context) else {
            panic!("{} is not a pointer", pointer.llvm_syntax());
        };

        emit!(self, "\t{} = alloca {}\n", pointer.llvm_syntax(), pointee_type.llvm_syntax(context))
    }

    pub fn emit_store(&mut self, value: &Value, pointer: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\tstore {} {}, {} {}\n",
            value.get_type().llvm_syntax(context),
            value.llvm_syntax(context),
            pointer.get_type().llvm_syntax(context),
            pointer.llvm_syntax(context),
        )
    }

    pub fn emit_load(&mut self, result: &Register, pointer: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = load {}, {} {}\n",
            result.llvm_syntax(),
            result.get_type().llvm_syntax(context),
            pointer.get_type().llvm_syntax(context),
            pointer.llvm_syntax(context),
        )
    }

    pub fn emit_get_element_pointer(&mut self, result: &Register, pointer: &Value, indices: &[Value], context: &GlobalContext) -> crate::Result<()> {
        let &TypeRepr::Pointer { pointee_type, .. } = pointer.get_type().repr(context) else {
            panic!("{} is not a pointer", pointer.llvm_syntax(context));
        };

        emit!(
            self,
            "\t{} = getelementptr inbounds {}, {} {}",
            result.llvm_syntax(),
            pointee_type.llvm_syntax(context),
            pointer.get_type().llvm_syntax(context),
            pointer.llvm_syntax(context),
        )?;

        for index in indices {
            emit!(
                self,
                ", {} {}",
                index.get_type().llvm_syntax(context),
                index.llvm_syntax(context),
            )?;
        }

        emit!(self, "\n")
    }

    pub fn emit_extract_value(&mut self, result: &Register, aggregate: &Value, indices: &[Value], context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = extractvalue {} {}",
            result.llvm_syntax(),
            aggregate.get_type().llvm_syntax(context),
            aggregate.llvm_syntax(context),
        )?;

        for index in indices {
            emit!(
                self,
                ", {} {}",
                index.get_type().llvm_syntax(context),
                index.llvm_syntax(context),
            )?;
        }

        emit!(self, "\n")
    }

    pub fn emit_insert_value(&mut self, result: &Register, aggregate: &Value, value: &Value, indices: &[Value], context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = insertvalue {} {}, {} {}",
            result.llvm_syntax(),
            aggregate.get_type().llvm_syntax(context),
            aggregate.llvm_syntax(context),
            value.get_type().llvm_syntax(context),
            value.llvm_syntax(context),
        )?;

        for index in indices {
            emit!(
                self,
                ", {} {}",
                index.get_type().llvm_syntax(context),
                index.llvm_syntax(context),
            )?;
        }

        emit!(self, "\n")
    }

    pub fn emit_bitwise_cast(&mut self, result: &Register, value: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = bitcast {} {} to {}\n",
            result.llvm_syntax(),
            value.get_type().llvm_syntax(context),
            value.llvm_syntax(context),
            result.get_type().llvm_syntax(context),
        )
    }

    pub fn emit_truncation(&mut self, result: &Register, value: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = trunc {} {} to {}\n",
            result.llvm_syntax(),
            value.get_type().llvm_syntax(context),
            value.llvm_syntax(context),
            result.get_type().llvm_syntax(context),
        )
    }

    pub fn emit_sign_extension(&mut self, result: &Register, value: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = sext {} {} to {}\n",
            result.llvm_syntax(),
            value.get_type().llvm_syntax(context),
            value.llvm_syntax(context),
            result.get_type().llvm_syntax(context),
        )
    }

    pub fn emit_zero_extension(&mut self, result: &Register, value: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = zext {} {} to {}\n",
            result.llvm_syntax(),
            value.get_type().llvm_syntax(context),
            value.llvm_syntax(context),
            result.get_type().llvm_syntax(context),
        )
    }

    pub fn emit_extension(&mut self, result: &Register, value: &Value, context: &GlobalContext) -> crate::Result<()> {
        match value.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => self.emit_sign_extension(result, value, context),
            _ => self.emit_zero_extension(result, value, context)
        }
    }

    pub fn emit_negation(&mut self, result: &Register, operand: &Value, context: &GlobalContext) -> crate::Result<()> {
        match operand.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = sub nsw {} 0, {}\n",
                result.llvm_syntax(),
                operand.get_type().llvm_syntax(context),
                operand.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = sub {} 0, {}\n",
                result.llvm_syntax(),
                operand.get_type().llvm_syntax(context),
                operand.llvm_syntax(context),
            )
        }
    }

    pub fn emit_addition(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = add nsw {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = add nuw {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_subtraction(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = sub nsw {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = sub nuw {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_multiplication(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = mul nsw {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = mul nuw {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_division(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = sdiv {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = udiv {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_remainder(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = srem {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = urem {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_shift_left(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = shl {} {}, {}\n",
            result.llvm_syntax(),
            lhs.get_type().llvm_syntax(context),
            lhs.llvm_syntax(context),
            rhs.llvm_syntax(context),
        )
    }

    pub fn emit_shift_right(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = ashr {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = lshr {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_bitwise_and(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = and {} {}, {}\n",
            result.llvm_syntax(),
            lhs.get_type().llvm_syntax(context),
            lhs.llvm_syntax(context),
            rhs.llvm_syntax(context),
        )
    }

    pub fn emit_bitwise_or(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = or {} {}, {}\n",
            result.llvm_syntax(),
            lhs.get_type().llvm_syntax(context),
            lhs.llvm_syntax(context),
            rhs.llvm_syntax(context),
        )
    }

    pub fn emit_bitwise_xor(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = xor {} {}, {}\n",
            result.llvm_syntax(),
            lhs.get_type().llvm_syntax(context),
            lhs.llvm_syntax(context),
            rhs.llvm_syntax(context),
        )
    }

    pub fn emit_inversion(&mut self, result: &Register, operand: &Value, context: &GlobalContext) -> crate::Result<()> {
        match operand.get_type().repr(context) {
            TypeRepr::Boolean => emit!(
                self,
                "\t{} = xor {} {}, true\n",
                result.llvm_syntax(),
                operand.get_type().llvm_syntax(context),
                operand.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = xor {} {}, -1\n",
                result.llvm_syntax(),
                operand.get_type().llvm_syntax(context),
                operand.llvm_syntax(context),
            )
        }
    }

    pub fn emit_cmp_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = icmp eq {} {}, {}\n",
            result.llvm_syntax(),
            lhs.get_type().llvm_syntax(context),
            lhs.llvm_syntax(context),
            rhs.llvm_syntax(context),
        )
    }

    pub fn emit_cmp_not_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = icmp ne {} {}, {}\n",
            result.llvm_syntax(),
            lhs.get_type().llvm_syntax(context),
            lhs.llvm_syntax(context),
            rhs.llvm_syntax(context),
        )
    }

    pub fn emit_cmp_less_than(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = icmp slt {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = icmp ult {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_cmp_less_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = icmp sle {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = icmp ule {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_cmp_greater_than(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = icmp sgt {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = icmp ugt {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_cmp_greater_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value, context: &GlobalContext) -> crate::Result<()> {
        match lhs.get_type().repr(context) {
            TypeRepr::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = icmp sge {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            ),
            _ => emit!(
                self,
                "\t{} = icmp uge {} {}, {}\n",
                result.llvm_syntax(),
                lhs.get_type().llvm_syntax(context),
                lhs.llvm_syntax(context),
                rhs.llvm_syntax(context),
            )
        }
    }

    pub fn emit_function_call(&mut self, result: Option<&Register>, callee: &Value, arguments: &[Value], context: &GlobalContext) -> crate::Result<()> {
        let TypeRepr::Function { signature } = callee.get_type().repr(context) else {
            panic!("{} is not a function", callee.llvm_syntax(context));
        };

        emit!(self, "\t")?;
        if let Some(result) = result {
            emit!(self, "{} = ", result.llvm_syntax())?;
        }
        emit!(self, "call {}(", signature.return_type().llvm_syntax(context))?;

        let mut parameters_iter = signature.parameter_types().iter();
        if let Some(&parameter_type) = parameters_iter.next() {
            emit!(self, "{}", parameter_type.llvm_syntax(context))?;
            for &parameter_type in parameters_iter {
                emit!(self, ", {}", parameter_type.llvm_syntax(context))?;
            }
            if signature.is_variadic() {
                emit!(self, ", ...")?;
            }
        }
        else if signature.is_variadic() {
            emit!(self, "...")?;
        }

        emit!(self, ") {}(", callee.llvm_syntax(context))?;

        let mut arguments_iter = arguments.iter();
        if let Some(argument) = arguments_iter.next() {
            emit!(self, "{} {}", argument.get_type().llvm_syntax(context), argument.llvm_syntax(context))?;
            for argument in arguments_iter {
                emit!(self, ", {} {}", argument.get_type().llvm_syntax(context), argument.llvm_syntax(context))?;
            }
        }

        emit!(self, ")\n")
    }

    pub fn emit_return(&mut self, value: &Value, context: &GlobalContext) -> crate::Result<()> {
        let value_type = value.get_type();
        if value_type == TypeHandle::VOID || value_type == TypeHandle::NEVER {
            emit!(self, "\tret void\n")
        }
        else {
            emit!(self, "\tret {} {}\n", value_type.llvm_syntax(context), value.llvm_syntax(context))
        }
    }

    pub fn emit_unreachable(&mut self) -> crate::Result<()> {
        emit!(self, "\tunreachable\n")
    }
}
