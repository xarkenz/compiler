use super::*;

use crate::sema::*;

use std::collections::{BTreeSet, BTreeMap};
use std::io::Write;

use indoc::writedoc;

macro_rules! emit {
    ($emitter:expr, $($args:tt)*) => {
        write!(($emitter).writer, $($args)*).map_err(|cause| ($emitter).error(cause))
    };
}

pub struct Emitter<W: Write> {
    filename: String,
    writer: W,
    is_global: bool,
    defined_functions: BTreeSet<Register>,
    queued_function_declarations: BTreeMap<Register, String>,
    defined_types: BTreeSet<String>,
    declared_types: BTreeSet<String>,
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
            defined_functions: BTreeSet::new(),
            queued_function_declarations: BTreeMap::new(),
            defined_types: BTreeSet::new(),
            declared_types: BTreeSet::new(),
            queued_global_declarations: Vec::new(),
        }
    }

    pub fn filename(&self) -> &str {
        self.filename.as_str()
    }

    fn error(&self, cause: std::io::Error) -> Box<crate::Error> {
        Box::new(crate::Error::OutputFileWrite { filename: self.filename.clone(), cause })
    }

    pub fn emit_preamble(&mut self, module_id: usize, source_filename: &str) -> crate::Result<()> {
        writedoc!(self.writer, "
            ; module_id = {module_id}
            source_filename = \"{source_filename}\"

        ").map_err(|cause| self.error(cause))
    }

    pub fn emit_postamble(&mut self) -> crate::Result<()> {
        // Write all queued type declarations
        for type_syntax in &self.declared_types {
            emit!(self, "{type_syntax} = type opaque\n\n")?;
        }
        // Write all queued function declarations
        for declaration in self.queued_function_declarations.values() {
            emit!(self, "{declaration}\n\n")?;
        }
        // Dequeue all declarations which were just written
        self.declared_types.clear();
        self.queued_function_declarations.clear();

        Ok(())
    }

    pub fn queue_function_declaration(&mut self, function: &Register, registry: &TypeRegistry) {
        if !self.defined_functions.contains(function) && !self.queued_function_declarations.contains_key(function) {
            let Some(signature) = registry.get_function_info(function.value_type()) else {
                panic!("{} is not a valid function", function.get_llvm_syntax());
            };
            let mut declaration = format!(
                "declare {} {}(",
                signature.return_type().get_llvm_syntax(registry),
                function.get_llvm_syntax(),
            );

            let mut parameters_iter = signature.parameter_types().iter();
            if let Some(&parameter_type) = parameters_iter.next() {
                declaration.push_str(parameter_type.get_llvm_syntax(registry));
                for &parameter_type in parameters_iter {
                    declaration.push_str(", ");
                    declaration.push_str(parameter_type.get_llvm_syntax(registry));
                }
                if signature.is_variadic() {
                    declaration.push_str(", ...");
                }
            }
            else if signature.is_variadic() {
                declaration.push_str("...");
            }
            
            declaration.push(')');

            // Enqueue the declaration which was just generated, which will be written before the module postamble
            // --that is, unless the function is defined later in the file
            self.queued_function_declarations.insert(function.clone(), declaration);
        }
    }

    pub fn emit_function_enter(&mut self, function: &Register, parameters: &[Register], registry: &TypeRegistry) -> crate::Result<()> {
        // Remove any forward declarations of this function from the declaration queue
        self.queued_function_declarations.remove(function);

        let Some(signature) = registry.get_function_info(function.value_type()) else {
            panic!("{} is not a valid function", function.get_llvm_syntax());
        };

        emit!(self, "define {} {}(", signature.return_type().get_llvm_syntax(registry), function.get_llvm_syntax())?;

        let mut parameters_iter = parameters.iter();
        if let Some(parameter) = parameters_iter.next() {
            emit!(self, "{} {}", parameter.value_type().get_llvm_syntax(registry), parameter.get_llvm_syntax())?;
            for parameter in parameters_iter {
                emit!(self, ", {} {}", parameter.value_type().get_llvm_syntax(registry), parameter.get_llvm_syntax())?;
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

    pub fn queue_type_declaration(&mut self, type_handle: TypeHandle, registry: &TypeRegistry) {
        let syntax = type_handle.get_llvm_syntax(registry);

        if !self.defined_types.contains(syntax) && !self.declared_types.contains(syntax) {
            // Enqueue the declaration, which will be written before the module postamble
            // --that is, unless the type is defined later in the file
            self.declared_types.insert(syntax.into());
        }
    }

    pub fn emit_type_definition(&mut self, type_handle: TypeHandle, registry: &TypeRegistry) -> crate::Result<()> {
        let syntax = type_handle.get_llvm_syntax(registry);

        // Remove any forward declarations of this type from the declaration queue
        self.declared_types.remove(syntax);

        let TypeInfo::Structure { members, .. } = registry.get_info(type_handle) else {
            panic!("{} is not a structure type", type_handle.get_identifier(registry));
        };
        
        emit!(self, "{syntax} = type ")?;
        let mut members_iter = members.iter();
        if let Some(&StructureMember { value_type, .. }) = members_iter.next() {
            emit!(self, "{{ {}", value_type.get_llvm_syntax(registry))?;
            for &StructureMember { value_type, .. } in members_iter {
                emit!(self, ", {}", value_type.get_llvm_syntax(registry))?;
            }
            emit!(self, " }}\n")
        }
        else {
            emit!(self, "{{}}\n")
        }
    }

    pub fn emit_global_allocation(&mut self, pointer: &Register, value: &Constant, is_constant: bool, registry: &TypeRegistry) -> crate::Result<()> {
        let declaration = format!(
            "{} = {} {} {}",
            pointer.get_llvm_syntax(),
            if is_constant { "constant" } else { "global" },
            value.get_type().get_llvm_syntax(registry),
            value.get_llvm_syntax(registry),
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

    pub fn emit_anonymous_constant(&mut self, pointer: &Register, value: &Constant, registry: &TypeRegistry) -> crate::Result<()> {
        let declaration = format!(
            "{} = private unnamed_addr constant {} {}",
            pointer.get_llvm_syntax(),
            value.get_type().get_llvm_syntax(registry),
            value.get_llvm_syntax(registry),
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
        let label_name = label.name();
        emit!(self, "{label_name}:\n")
    }

    pub fn emit_unconditional_branch(&mut self, label: &Label) -> crate::Result<()> {
        emit!(self, "\tbr label {label}\n")
    }

    pub fn emit_conditional_branch(&mut self, condition: &Value, consequent: &Label, alternative: &Label, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(self, "\tbr i1 {}, label {consequent}, label {alternative}\n", condition.get_llvm_syntax(registry))
    }

    pub fn emit_local_allocation(&mut self, pointer: &Register, registry: &TypeRegistry) -> crate::Result<()> {
        let &TypeInfo::Pointer { pointee_type, .. } = registry.get_info(pointer.value_type()) else {
            panic!("{} is not a pointer", pointer.get_llvm_syntax());
        };
        
        emit!(self, "\t{} = alloca {}\n", pointer.get_llvm_syntax(), pointee_type.get_llvm_syntax(registry))
    }

    pub fn emit_store(&mut self, value: &Value, pointer: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\tstore {} {}, {} {}\n",
            value.get_type().get_llvm_syntax(registry),
            value.get_llvm_syntax(registry),
            pointer.get_type().get_llvm_syntax(registry),
            pointer.get_llvm_syntax(registry),
        )
    }

    pub fn emit_load(&mut self, result: &Register, pointer: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = load {}, {} {}\n",
            result.get_llvm_syntax(),
            result.value_type().get_llvm_syntax(registry),
            pointer.get_type().get_llvm_syntax(registry),
            pointer.get_llvm_syntax(registry),
        )
    }

    pub fn emit_get_element_pointer(&mut self, result: &Register, pointer: &Value, indices: &[Value], registry: &TypeRegistry) -> crate::Result<()> {
        let &TypeInfo::Pointer { pointee_type, .. } = registry.get_info(pointer.get_type()) else {
            panic!("{} is not a pointer", pointer.get_llvm_syntax(registry));
        };
        
        emit!(
            self,
            "\t{} = getelementptr inbounds {}, {} {}",
            result.get_llvm_syntax(),
            pointee_type.get_llvm_syntax(registry),
            pointer.get_type().get_llvm_syntax(registry),
            pointer.get_llvm_syntax(registry),
        )?;

        for index in indices {
            emit!(
                self,
                ", {} {}",
                index.get_type().get_llvm_syntax(registry),
                index.get_llvm_syntax(registry),
            )?;
        }

        emit!(self, "\n")
    }

    pub fn emit_extract_value(&mut self, result: &Register, aggregate: &Value, indices: &[Value], registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = extractvalue {} {}",
            result.get_llvm_syntax(),
            aggregate.get_type().get_llvm_syntax(registry),
            aggregate.get_llvm_syntax(registry),
        )?;

        for index in indices {
            emit!(
                self,
                ", {} {}",
                index.get_type().get_llvm_syntax(registry),
                index.get_llvm_syntax(registry),
            )?;
        }

        emit!(self, "\n")
    }

    pub fn emit_insert_value(&mut self, result: &Register, aggregate: &Value, value: &Value, indices: &[Value], registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = insertvalue {} {}, {} {}",
            result.get_llvm_syntax(),
            aggregate.get_type().get_llvm_syntax(registry),
            aggregate.get_llvm_syntax(registry),
            value.get_type().get_llvm_syntax(registry),
            value.get_llvm_syntax(registry),
        )?;

        for index in indices {
            emit!(
                self,
                ", {} {}",
                index.get_type().get_llvm_syntax(registry),
                index.get_llvm_syntax(registry),
            )?;
        }

        emit!(self, "\n")
    }

    pub fn emit_bitwise_cast(&mut self, result: &Register, value: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = bitcast {} {} to {}\n",
            result.get_llvm_syntax(),
            value.get_type().get_llvm_syntax(registry),
            value.get_llvm_syntax(registry),
            result.value_type().get_llvm_syntax(registry),
        )
    }

    pub fn emit_truncation(&mut self, result: &Register, value: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = trunc {} {} to {}\n",
            result.get_llvm_syntax(),
            value.get_type().get_llvm_syntax(registry),
            value.get_llvm_syntax(registry),
            result.value_type().get_llvm_syntax(registry),
        )
    }

    pub fn emit_sign_extension(&mut self, result: &Register, value: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = sext {} {} to {}\n",
            result.get_llvm_syntax(),
            value.get_type().get_llvm_syntax(registry),
            value.get_llvm_syntax(registry),
            result.value_type().get_llvm_syntax(registry),
        )
    }

    pub fn emit_zero_extension(&mut self, result: &Register, value: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = zext {} {} to {}\n",
            result.get_llvm_syntax(),
            value.get_type().get_llvm_syntax(registry),
            value.get_llvm_syntax(registry),
            result.value_type().get_llvm_syntax(registry),
        )
    }

    pub fn emit_extension(&mut self, result: &Register, value: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(value.get_type()) {
            TypeInfo::Integer { signed: true, .. } => self.emit_sign_extension(result, value, registry),
            _ => self.emit_zero_extension(result, value, registry)
        }
    }

    pub fn emit_negation(&mut self, result: &Register, operand: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(operand.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = sub nsw {} 0, {}\n",
                result.get_llvm_syntax(),
                operand.get_type().get_llvm_syntax(registry),
                operand.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = sub {} 0, {}\n",
                result.get_llvm_syntax(),
                operand.get_type().get_llvm_syntax(registry),
                operand.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_addition(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = add nsw {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = add nuw {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_subtraction(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = sub nsw {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = sub nuw {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_multiplication(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = mul nsw {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = mul nuw {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_division(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = sdiv {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = udiv {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_remainder(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = srem {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = urem {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_shift_left(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = shl {} {}, {}\n",
            result.get_llvm_syntax(),
            lhs.get_type().get_llvm_syntax(registry),
            lhs.get_llvm_syntax(registry),
            rhs.get_llvm_syntax(registry),
        )
    }

    pub fn emit_shift_right(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = ashr {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = lshr {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_bitwise_and(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = and {} {}, {}\n",
            result.get_llvm_syntax(),
            lhs.get_type().get_llvm_syntax(registry),
            lhs.get_llvm_syntax(registry),
            rhs.get_llvm_syntax(registry),
        )
    }

    pub fn emit_bitwise_or(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = or {} {}, {}\n",
            result.get_llvm_syntax(),
            lhs.get_type().get_llvm_syntax(registry),
            lhs.get_llvm_syntax(registry),
            rhs.get_llvm_syntax(registry),
        )
    }

    pub fn emit_bitwise_xor(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = xor {} {}, {}\n",
            result.get_llvm_syntax(),
            lhs.get_type().get_llvm_syntax(registry),
            lhs.get_llvm_syntax(registry),
            rhs.get_llvm_syntax(registry),
        )
    }

    pub fn emit_inversion(&mut self, result: &Register, operand: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(operand.get_type()) {
            TypeInfo::Boolean => emit!(
                self,
                "\t{} = xor {} {}, true\n",
                result.get_llvm_syntax(),
                operand.get_type().get_llvm_syntax(registry),
                operand.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = xor {} {}, -1\n",
                result.get_llvm_syntax(),
                operand.get_type().get_llvm_syntax(registry),
                operand.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_cmp_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = icmp eq {} {}, {}\n",
            result.get_llvm_syntax(),
            lhs.get_type().get_llvm_syntax(registry),
            lhs.get_llvm_syntax(registry),
            rhs.get_llvm_syntax(registry),
        )
    }

    pub fn emit_cmp_not_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        emit!(
            self,
            "\t{} = icmp ne {} {}, {}\n",
            result.get_llvm_syntax(),
            lhs.get_type().get_llvm_syntax(registry),
            lhs.get_llvm_syntax(registry),
            rhs.get_llvm_syntax(registry),
        )
    }

    pub fn emit_cmp_less_than(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = icmp slt {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = icmp ult {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_cmp_less_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = icmp sle {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = icmp ule {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_cmp_greater_than(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = icmp sgt {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = icmp ugt {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_cmp_greater_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value, registry: &TypeRegistry) -> crate::Result<()> {
        match registry.get_info(lhs.get_type()) {
            TypeInfo::Integer { signed: true, .. } => emit!(
                self,
                "\t{} = icmp sge {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            ),
            _ => emit!(
                self,
                "\t{} = icmp uge {} {}, {}\n",
                result.get_llvm_syntax(),
                lhs.get_type().get_llvm_syntax(registry),
                lhs.get_llvm_syntax(registry),
                rhs.get_llvm_syntax(registry),
            )
        }
    }

    pub fn emit_function_call(&mut self, result: Option<&Register>, callee: &Value, arguments: &[Value], registry: &TypeRegistry) -> crate::Result<()> {
        let TypeInfo::Function { signature } = registry.get_info(callee.get_type()) else {
            panic!("{} is not a function", callee.get_llvm_syntax(registry));
        };

        emit!(self, "\t")?;
        if let Some(result) = result {
            emit!(self, "{} = ", result.get_llvm_syntax())?;
        }
        emit!(self, "\tcall {}(", signature.return_type().get_llvm_syntax(registry))?;
        
        let mut parameters_iter = signature.parameter_types().iter();
        if let Some(&parameter_type) = parameters_iter.next() {
            emit!(self, "{}", parameter_type.get_llvm_syntax(registry))?;
            for &parameter_type in parameters_iter {
                emit!(self, ", {}", parameter_type.get_llvm_syntax(registry))?;
            }
            if signature.is_variadic() {
                emit!(self, ", ...")?;
            }
        }
        else if signature.is_variadic() {
            emit!(self, "...")?;
        }
        
        emit!(self, ") {}(", callee.get_llvm_syntax(registry))?;

        let mut arguments_iter = arguments.iter();
        if let Some(argument) = arguments_iter.next() {
            emit!(self, "{} {}", argument.get_type().get_llvm_syntax(registry), argument.get_llvm_syntax(registry))?;
            for argument in arguments_iter {
                emit!(self, ", {} {}", argument.get_type().get_llvm_syntax(registry), argument.get_llvm_syntax(registry))?;
            }
        }

        emit!(self, ")\n")
    }

    pub fn emit_return(&mut self, value: Option<&Value>, registry: &TypeRegistry) -> crate::Result<()> {
        if let Some(value) = value {
            emit!(self, "\tret {} {}\n", value.get_type().get_llvm_syntax(registry), value.get_llvm_syntax(registry))
        }
        else {
            emit!(self, "\tret void\n")
        }
    }

    pub fn emit_unreachable(&mut self) -> crate::Result<()> {
        emit!(self, "\tunreachable\n")
    }
}
