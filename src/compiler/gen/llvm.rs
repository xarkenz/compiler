use super::*;

use std::io::Write;

use indoc::writedoc;

const INDENT: &str = "    ";

pub fn emit_preamble<T: Write>(emitter: &mut T, source_filename: &str) -> std::io::Result<()> {
    writedoc!(emitter, "
        ; ModuleID = '{source_filename}'
        ; source_filename = \"{source_filename}\"

        target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"
        target triple = \"x86_64-pc-linux-gnu\"

        @print_i64_fstring = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\", align 1
        @print_u64_fstring = private unnamed_addr constant [6 x i8] c\"%llu\\0A\\00\", align 1
        @print_ptr_fstring = private unnamed_addr constant [4 x i8] c\"%p\\0A\\00\", align 1

        define dso_local i32 @main() #0 {{
    ")
}

pub fn emit_label<T: Write>(emitter: &mut T, label: &Label) -> std::io::Result<()> {
    write!(
        emitter,
        "{label_name}:\n",
        label_name = label.name(),
    )
}

pub fn emit_unconditional_branch<T: Write>(emitter: &mut T, label: &Label) -> std::io::Result<()> {
    write!(
        emitter,
        "{INDENT}br label {label}\n",
    )
}

pub fn emit_conditional_branch<T: Write>(emitter: &mut T, condition: &RightValue, consequent: &Label, alternative: &Label) -> std::io::Result<()> {
    write!(
        emitter,
        "{INDENT}br i1 {condition}, label {consequent}, label {alternative}\n",
    )
}

pub fn emit_symbol_declaration<T: Write>(emitter: &mut T, symbol: &info::Symbol) -> std::io::Result<()> {
    if symbol.register().is_global() {
        write!(
            emitter,
            "{register} = global {format}, align {alignment}\n",
            register = symbol.register(),
            format = symbol.format(),
            alignment = symbol.alignment(),
        )
    }
    else {
        write!(
            emitter,
            "{INDENT}{register} = alloca {format}, align {alignment}\n",
            register = symbol.register(),
            format = symbol.format(),
            alignment = symbol.alignment(),
        )
    }
}

pub fn emit_symbol_store<T: Write>(emitter: &mut T, value: &RightValue, symbol: &info::Symbol) -> std::io::Result<()> {
    write!(
        emitter,
        "{INDENT}store {value_format} {value}, {symbol_register_format} {symbol_register}\n",
        value_format = value.format(),
        symbol_register_format = symbol.register().format(),
        symbol_register = symbol.register(),
    )
}

pub fn emit_symbol_load<T: Write>(emitter: &mut T, register: &Register, symbol: &info::Symbol) -> std::io::Result<()> {
    write!(
        emitter,
        "{INDENT}{register} = load {register_format}, {symbol_register_format} {symbol_register}\n",
        register_format = register.format(),
        symbol_register_format = symbol.register().format(),
        symbol_register = symbol.register(),
    )
}

pub fn emit_truncation<T: Write>(emitter: &mut T, result: &Register, value: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "{INDENT}{result} = trunc {from_format} {value} to {to_format}\n",
        to_format = result.format(),
        from_format = value.format(),
    )
}

pub fn emit_sign_extension<T: Write>(emitter: &mut T, result: &Register, value: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "{INDENT}{result} = sext {from_format} {value} to {to_format}\n",
        to_format = result.format(),
        from_format = value.format(),
    )
}

pub fn emit_zero_extension<T: Write>(emitter: &mut T, result: &Register, value: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "{INDENT}{result} = zext {from_format} {value} to {to_format}\n",
        to_format = result.format(),
        from_format = value.format(),
    )
}

pub fn emit_extension<T: Write>(emitter: &mut T, result: &Register, value: &RightValue) -> std::io::Result<()> {
    match value.format() {
        ValueFormat::Integer { signed: true, .. } => {
            emit_sign_extension(emitter, result, value)
        },
        _ => {
            emit_zero_extension(emitter, result, value)
        }
    }
}

pub fn emit_addition<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    match lhs.format() {
        ValueFormat::Integer { signed: true, .. } => write!(
            emitter,
            "{INDENT}{result} = add nsw {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        ),
        _ => write!(
            emitter,
            "{INDENT}{result} = add nuw {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        )
    }
}

pub fn emit_subtraction<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    match lhs.format() {
        ValueFormat::Integer { signed: true, .. } => write!(
            emitter,
            "{INDENT}{result} = sub nsw {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        ),
        _ => write!(
            emitter,
            "{INDENT}{result} = sub nuw {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        )
    }
}

pub fn emit_multiplication<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    match lhs.format() {
        ValueFormat::Integer { signed: true, .. } => write!(
            emitter,
            "{INDENT}{result} = mul nsw {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        ),
        _ => write!(
            emitter,
            "{INDENT}{result} = mul nuw {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        )
    }
}

pub fn emit_division<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    match lhs.format() {
        ValueFormat::Integer { signed: true, .. } => write!(
            emitter,
            "{INDENT}{result} = sdiv {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        ),
        _ => write!(
            emitter,
            "{INDENT}{result} = udiv {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        )
    }
}

pub fn emit_cmp_equal<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "{INDENT}{result} = icmp eq {format} {lhs}, {rhs}\n",
        format = lhs.format(),
    )
}

pub fn emit_cmp_not_equal<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "{INDENT}{result} = icmp ne {format} {lhs}, {rhs}\n",
        format = lhs.format(),
    )
}

pub fn emit_cmp_less_than<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    match lhs.format() {
        ValueFormat::Integer { signed: true, .. } => write!(
            emitter,
            "{INDENT}{result} = icmp slt {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        ),
        _ => write!(
            emitter,
            "{INDENT}{result} = icmp ult {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        )
    }
}

pub fn emit_cmp_less_equal<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    match lhs.format() {
        ValueFormat::Integer { signed: true, .. } => write!(
            emitter,
            "{INDENT}{result} = icmp sle {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        ),
        _ => write!(
            emitter,
            "{INDENT}{result} = icmp ule {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        )
    }
}

pub fn emit_cmp_greater_than<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    match lhs.format() {
        ValueFormat::Integer { signed: true, .. } => write!(
            emitter,
            "{INDENT}{result} = icmp sgt {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        ),
        _ => write!(
            emitter,
            "{INDENT}{result} = icmp ugt {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        )
    }
}

pub fn emit_cmp_greater_equal<T: Write>(emitter: &mut T, result: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    match lhs.format() {
        ValueFormat::Integer { signed: true, .. } => write!(
            emitter,
            "{INDENT}{result} = icmp sge {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        ),
        _ => write!(
            emitter,
            "{INDENT}{result} = icmp uge {format} {lhs}, {rhs}\n",
            format = lhs.format(),
        )
    }
}

pub fn emit_print<T: Write>(emitter: &mut T, result: &Register, value: &RightValue) -> std::io::Result<()> {
    match value.format() {
        ValueFormat::Integer { signed: true, .. } => write!(
            emitter,
            "{INDENT}{result} = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_i64_fstring, i32 0, i32 0), i64 {value})\n",
        ),
        ValueFormat::Integer { signed: false, .. } => write!(
            emitter,
            "{INDENT}{result} = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @print_u64_fstring, i32 0, i32 0), i64 {value})\n",
        ),
        _ => write!(
            emitter,
            "{INDENT}{result} = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @print_ptr_fstring, i32 0, i32 0), ptr {value})\n",
        )
    }
}

pub fn emit_return<T: Write>(emitter: &mut T, value: Option<&RightValue>) -> std::io::Result<()> {
    if let Some(value) = value {
        write!(
            emitter,
            "{INDENT}ret {format} {value}\n",
            format = value.format(),
        )
    } else {
        write!(
            emitter,
            "{INDENT}ret void\n",
        )
    }
}

pub fn emit_postamble<T: Write>(emitter: &mut T) -> std::io::Result<()> {
    writedoc!(emitter, "
        }}

        declare i32 @printf(i8*, ...) #1

        attributes #0 = {{
        {INDENT}noinline nounwind optnone uwtable
        {INDENT}\"frame-pointer\"=\"all\"
        {INDENT}\"min-legal-vector-width\"=\"0\"
        {INDENT}\"no-trapping-math\"=\"true\"
        {INDENT}\"stack-protector-buffer-size\"=\"8\"
        {INDENT}\"target-cpu\"=\"x86-64\"
        {INDENT}\"target-features\"=\"+cx8,+fxsr,+mmx,+sse,+sse2,+x87\"
        {INDENT}\"tune-cpu\"=\"generic\"
        }}
        attributes #1 = {{
        {INDENT}\"frame-pointer\"=\"all\"
        {INDENT}\"no-trapping-math\"=\"true\"
        {INDENT}\"stack-protector-buffer-size\"=\"8\"
        {INDENT}\"target-cpu\"=\"x86-64\"
        {INDENT}\"target-features\"=\"+cx8,+fxsr,+mmx,+sse,+sse2,+x87\"
        {INDENT}\"tune-cpu\"=\"generic\"
        }}

        !llvm.module.flags = !{{!0, !1, !2, !3, !4}}
        !llvm.ident = !{{!5}}
        !0 = !{{i32 1, !\"wchar_size\", i32 4}}
        !1 = !{{i32 7, !\"PIC Level\", i32 2}}
        !2 = !{{i32 7, !\"PIE Level\", i32 2}}
        !3 = !{{i32 7, !\"uwtable\", i32 1}}
        !4 = !{{i32 7, !\"frame-pointer\", i32 2}}
        !5 = !{{!\"Ubuntu clang version 10.0.0-4ubuntu1\"}}
    ")
}
