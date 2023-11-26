use super::*;

use std::io::Write;

use indoc::writedoc;

pub struct Emitter<W: Write> {
    name: String,
    writer: W,
    used_attribute_group_0: bool,
    used_attribute_group_1: bool,
    defined_functions: Vec<Register>,
    queued_function_declarations: Vec<(Register, String)>,
}

impl Emitter<std::fs::File> {
    pub fn from_filename(filename: String) -> crate::Result<Self> {
        std::fs::File::create(&filename)
            .map(|file| Self::new(filename.clone(), file))
            .map_err(|cause| crate::FileError::new(filename, None, cause).into_boxed())
    }
}

impl<W: Write> Emitter<W> {
    pub fn new(name: String, writer: W) -> Self {
        Self {
            name,
            writer,
            used_attribute_group_0: false,
            used_attribute_group_1: false,
            defined_functions: Vec::new(),
            queued_function_declarations: Vec::new(),
        }
    }

    fn error(&self, cause: std::io::Error) -> Box<dyn crate::Error> {
        crate::FileError::new(self.name.clone(), None, cause).into_boxed()
    }

    pub fn emit_preamble(&mut self, source_filename: &str) -> crate::Result<()> {
        writedoc!(self.writer, "
            source_filename = \"{source_filename}\"

            target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"
            target triple = \"x86_64-pc-linux-gnu\"

            @print_i64_fstring = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\", align 1
            @print_u64_fstring = private unnamed_addr constant [6 x i8] c\"%llu\\0A\\00\", align 1
            @print_ptr_fstring = private unnamed_addr constant [4 x i8] c\"%p\\0A\\00\", align 1

        ")
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_postamble(&mut self) -> crate::Result<()> {
        // Any emitted declarations will use attributes #1
        if !self.queued_function_declarations.is_empty() {
            self.used_attribute_group_1 = true;
        }
        // Write all function declarations queued for writing
        for (_, declaration) in &self.queued_function_declarations {
            writeln!(self.writer, "{declaration}\n")
                .map_err(|cause| self.error(cause))?;
        }
        // Dequeue all declarations which were just written
        self.queued_function_declarations.clear();
        self.queued_function_declarations.shrink_to_fit();

        if self.used_attribute_group_0 {
            writedoc!(self.writer, "
                attributes #0 = {{
                \tnoinline nounwind optnone uwtable
                \t\"frame-pointer\"=\"all\"
                \t\"min-legal-vector-width\"=\"0\"
                \t\"no-trapping-math\"=\"true\"
                \t\"stack-protector-buffer-size\"=\"8\"
                \t\"target-cpu\"=\"x86-64\"
                \t\"target-features\"=\"+cx8,+fxsr,+mmx,+sse,+sse2,+x87\"
                \t\"tune-cpu\"=\"generic\"
                }}

            ")
            .map_err(|cause| self.error(cause))?;
        }
        if self.used_attribute_group_1 {
            writedoc!(self.writer, "
                attributes #1 = {{
                \t\"frame-pointer\"=\"all\"
                \t\"no-trapping-math\"=\"true\"
                \t\"stack-protector-buffer-size\"=\"8\"
                \t\"target-cpu\"=\"x86-64\"
                \t\"target-features\"=\"+cx8,+fxsr,+mmx,+sse,+sse2,+x87\"
                \t\"tune-cpu\"=\"generic\"
                }}

            ")
            .map_err(|cause| self.error(cause))?;
        }

        writedoc!(self.writer, "
            !llvm.module.flags = !{{ !0, !1, !2, !3, !4 }}
            !llvm.ident = !{{ !5 }}
            !0 = !{{ i32 1, !\"wchar_size\", i32 4 }}
            !1 = !{{ i32 7, !\"PIC Level\", i32 2 }}
            !2 = !{{ i32 7, !\"PIE Level\", i32 2 }}
            !3 = !{{ i32 7, !\"uwtable\", i32 1 }}
            !4 = !{{ i32 7, !\"frame-pointer\", i32 2 }}
            !5 = !{{ !\"Ubuntu clang version 14.0.0-1ubuntu1.1\" }}
        ")
        .map_err(|cause| self.error(cause))
    }

    pub fn queue_function_declaration(&mut self, function: &Register, return_format: &ValueFormat, parameter_formats: &[ValueFormat], is_varargs: bool) -> crate::Result<()> {
        if !self.defined_functions.contains(function) && (
            !self.queued_function_declarations.iter().any(|(declared_function, _)| declared_function == function)
        ) {
            // Not a fan of .unwrap() spam, but write!() on a String shouldn't fail
            use std::fmt::Write;
            let mut declaration = String::new();

            write!(declaration, "declare {return_format} {function}(").unwrap();

            let mut parameter_formats = parameter_formats.iter();
            if let Some(format) = parameter_formats.next() {
                write!(declaration, "{format} noundef").unwrap();
                for format in parameter_formats {
                    write!(declaration, ", {format} noundef").unwrap();
                }
                if is_varargs {
                    write!(declaration, ", ...").unwrap();
                }
            }
            else if is_varargs {
                write!(declaration, "...").unwrap();
            }
            
            write!(declaration, ") #1").unwrap();

            // Enqueue the declaration which was just generated, which will be written before the module postamble
            // --that is, unless the function is defined later in the file
            self.queued_function_declarations.push((function.clone(), declaration));
        }

        Ok(())
    }

    pub fn emit_function_enter(&mut self, function: &Register, return_format: &ValueFormat, parameters: &[Register], is_varargs: bool) -> crate::Result<()> {
        // Remove any forward declarations of this function from the declaration queue
        self.queued_function_declarations.retain(|(declared_function, _)| declared_function.name() != function.name());

        write!(self.writer, "define dso_local {return_format} {function}(")
            .map_err(|cause| self.error(cause))?;

        let mut parameters = parameters.iter();
        if let Some(parameter) = parameters.next() {
            write!(self.writer, "{format} noundef {parameter}", format = parameter.format())
                .map_err(|cause| self.error(cause))?;
            for parameter in parameters {
                write!(self.writer, ", {format} noundef {parameter}", format = parameter.format())
                    .map_err(|cause| self.error(cause))?;
            }
            if is_varargs {
                write!(self.writer, ", ...")
                    .map_err(|cause| self.error(cause))?;
            }
        }
        else if is_varargs {
            write!(self.writer, "...")
                .map_err(|cause| self.error(cause))?;
        }

        // All emitted function definitions use attributes #0
        self.used_attribute_group_0 = true;
        
        writeln!(self.writer, ") #0 {{")
            .map_err(|cause| self.error(cause))
    }

    pub fn emit_function_exit(&mut self) -> crate::Result<()> {
        writeln!(
            self.writer,
            "}}\n",
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_label(&mut self, label: &Label) -> crate::Result<()> {
        writeln!(
            self.writer,
            "{label_name}:",
            label_name = label.name(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_unconditional_branch(&mut self, label: &Label) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\tbr label {label}",
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_conditional_branch(&mut self, condition: &Value, consequent: &Label, alternative: &Label) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\tbr i1 {condition}, label {consequent}, label {alternative}",
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_global_allocation(&mut self, pointer: &Register, init_value: &Constant, constant: bool) -> crate::Result<()> {
        if constant {
            writeln!(
                self.writer,
                "{pointer} = dso_local constant {format} {init_value}\n",
                format = init_value.format(),
            )
        }
        else {
            writeln!(
                self.writer,
                "{pointer} = dso_local global {format} {init_value}\n",
                format = init_value.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_local_allocation(&mut self, pointer: &Register, format: &ValueFormat) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{pointer} = alloca {format}",
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_store(&mut self, value: &Value, pointer: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\tstore {value_format} {value}, {pointer_format} {pointer}",
            value_format = value.format(),
            pointer_format = pointer.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_load(&mut self, result: &Register, pointer: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = load {result_format}, {pointer_format} {pointer}",
            result_format = result.format(),
            pointer_format = pointer.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_truncation(&mut self, result: &Register, value: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = trunc {from_format} {value} to {to_format}",
            to_format = result.format(),
            from_format = value.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_sign_extension(&mut self, result: &Register, value: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = sext {from_format} {value} to {to_format}",
            to_format = result.format(),
            from_format = value.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_zero_extension(&mut self, result: &Register, value: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = zext {from_format} {value} to {to_format}",
            to_format = result.format(),
            from_format = value.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_extension(&mut self, result: &Register, value: &Value) -> crate::Result<()> {
        match value.format() {
            ValueFormat::Integer { signed: true, .. } => self.emit_sign_extension(result, value),
            _ => self.emit_zero_extension(result, value)
        }
    }

    pub fn emit_negation(&mut self, result: &Register, operand: &Value) -> crate::Result<()> {
        match operand.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = sub nsw {format} 0, {operand}",
                format = operand.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = sub {format} 0, {operand}",
                format = operand.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_addition(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = add nsw {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = add nuw {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_subtraction(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = sub nsw {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = sub nuw {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_multiplication(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = mul nsw {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = mul nuw {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_division(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = sdiv {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = udiv {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_remainder(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = srem {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = urem {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_shift_left(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = shl {format} {lhs}, {rhs}",
            format = lhs.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_shift_right(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = ashr {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = lshr {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_bitwise_and(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = and {format} {lhs}, {rhs}",
            format = lhs.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_bitwise_or(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = or {format} {lhs}, {rhs}",
            format = lhs.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_bitwise_xor(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = xor {format} {lhs}, {rhs}",
            format = lhs.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_inversion(&mut self, result: &Register, operand: &Value) -> crate::Result<()> {
        match operand.format() {
            ValueFormat::Boolean => writeln!(
                self.writer,
                "\t{result} = xor {format} {operand}, true",
                format = operand.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = xor {format} {operand}, -1",
                format = operand.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_cmp_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = icmp eq {format} {lhs}, {rhs}",
            format = lhs.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_cmp_not_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\t{result} = icmp ne {format} {lhs}, {rhs}",
            format = lhs.format(),
        )
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_cmp_less_than(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = icmp slt {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = icmp ult {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_cmp_less_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = icmp sle {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = icmp ule {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_cmp_greater_than(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = icmp sgt {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = icmp ugt {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_cmp_greater_equal(&mut self, result: &Register, lhs: &Value, rhs: &Value) -> crate::Result<()> {
        match lhs.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = icmp sge {format} {lhs}, {rhs}",
                format = lhs.format(),
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = icmp uge {format} {lhs}, {rhs}",
                format = lhs.format(),
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_function_call(&mut self, result: Option<&Register>, callee: &Value, arguments: &[Value]) -> crate::Result<()> {
        if let Some(result) = result {
            write!(
                self.writer,
                "\t{result} = call {callee_format} {callee}(",
                callee_format = callee.format(),
            )
        }
        else {
            write!(
                self.writer,
                "\tcall {callee_format} {callee}(",
                callee_format = callee.format(),
            )
        }
        .map_err(|cause| self.error(cause))?;

        let mut arguments_iter = arguments.iter();
        if let Some(first) = arguments_iter.next() {
            write!(self.writer, "{format} noundef {first}", format = first.format())
                .map_err(|cause| self.error(cause))?;
            for argument in arguments_iter {
                write!(self.writer, ", {format} noundef {argument}", format = argument.format())
                    .map_err(|cause| self.error(cause))?;
            }
        }

        writeln!(self.writer, ")")
            .map_err(|cause| self.error(cause))
    }

    pub fn emit_print(&mut self, result: &Register, value: &Value) -> crate::Result<()> {
        match value.format() {
            ValueFormat::Integer { signed: true, .. } => writeln!(
                self.writer,
                "\t{result} = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([6 x i8], [6 x i8]* @print_i64_fstring, i32 0, i32 0), i64 noundef {value})",
            ),
            ValueFormat::Integer { signed: false, .. } => writeln!(
                self.writer,
                "\t{result} = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([6 x i8], [6 x i8]* @print_u64_fstring, i32 0, i32 0), i64 noundef {value})",
            ),
            _ => writeln!(
                self.writer,
                "\t{result} = call i32(i8*, ...) @printf(i8* noundef getelementptr inbounds ([4 x i8], [4 x i8]* @print_ptr_fstring, i32 0, i32 0), ptr noundef {value})",
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_return(&mut self, value: Option<&Value>) -> crate::Result<()> {
        if let Some(value) = value {
            writeln!(
                self.writer,
                "\tret {format} {value}",
                format = value.format(),
            )
        }
        else {
            writeln!(
                self.writer,
                "\tret void",
            )
        }
        .map_err(|cause| self.error(cause))
    }

    pub fn emit_unreachable(&mut self) -> crate::Result<()> {
        writeln!(
            self.writer,
            "\tunreachable",
        )
        .map_err(|cause| self.error(cause))
    }
}
