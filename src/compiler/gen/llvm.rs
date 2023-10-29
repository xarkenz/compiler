use super::*;

use std::io::Write;

use indoc::writedoc;

pub fn emit_preamble<T: Write>(emitter: &mut T, source_filename: &str) -> std::io::Result<()> {
    writedoc!(emitter, "
        ; ModuleID = '{source_filename}'
        ; source_filename = \"{source_filename}\"

        target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"
        target triple = \"x86_64-pc-linux-gnu\"

        @print_int_fstring = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\", align 1

        define dso_local i32 @main() #0 {{
    ")
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
            "    {register} = alloca {format}, align {alignment}\n",
            register = symbol.register(),
            format = symbol.format(),
            alignment = symbol.alignment(),
        )
    }
}

pub fn emit_symbol_store<T: Write>(emitter: &mut T, value: &RightValue, symbol: &info::Symbol) -> std::io::Result<()> {
    write!(
        emitter,
        "    store {value_format} {value}, {symbol_register_format} {symbol_register}\n",
        value_format = value.format(),
        symbol_register_format = symbol.register().format(),
        symbol_register = symbol.register(),
    )
}

pub fn emit_symbol_load<T: Write>(emitter: &mut T, register: &Register, symbol: &info::Symbol) -> std::io::Result<()> {
    write!(
        emitter,
        "    {register} = load {register_format}, {symbol_register_format} {symbol_register}\n",
        register_format = register.format(),
        symbol_register_format = symbol.register().format(),
        symbol_register = symbol.register(),
    )
}

pub fn emit_addition<T: Write>(emitter: &mut T, output: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "    {output} = add nsw {format} {lhs}, {rhs}\n",
        format = output.format(),
    )
}

pub fn emit_subtraction<T: Write>(emitter: &mut T, output: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "    {output} = sub nsw {format} {lhs}, {rhs}\n",
        format = output.format(),
    )
}

pub fn emit_multiplication<T: Write>(emitter: &mut T, output: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "    {output} = mul nsw {format} {lhs}, {rhs}\n",
        format = output.format(),
    )
}

pub fn emit_division<T: Write>(emitter: &mut T, output: &Register, lhs: &RightValue, rhs: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "    {output} = sdiv {format} {lhs}, {rhs}\n",
        format = output.format(),
    )
}

pub fn emit_print_i32<T: Write>(emitter: &mut T, output: &Register, input: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "    {output} = call i32(i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @print_int_fstring, i32 0, i32 0), {input_format} {input})\n",
        input_format = input.format(),
    )
}

pub fn emit_return<T: Write>(emitter: &mut T, value: &RightValue) -> std::io::Result<()> {
    write!(
        emitter,
        "    ret {format} {value}\n",
        format = value.format(),
    )
}

pub fn emit_postamble<T: Write>(emitter: &mut T) -> std::io::Result<()> {
    writedoc!(emitter, "
        }}

        declare i32 @printf(i8*, ...) #1

        attributes #0 = {{
            noinline nounwind optnone uwtable
            \"frame-pointer\"=\"all\"
            \"min-legal-vector-width\"=\"0\"
            \"no-trapping-math\"=\"true\"
            \"stack-protector-buffer-size\"=\"8\"
            \"target-cpu\"=\"x86-64\"
            \"target-features\"=\"+cx8,+fxsr,+mmx,+sse,+sse2,+x87\"
            \"tune-cpu\"=\"generic\"
        }}
        attributes #1 = {{
            \"frame-pointer\"=\"all\"
            \"no-trapping-math\"=\"true\"
            \"stack-protector-buffer-size\"=\"8\"
            \"target-cpu\"=\"x86-64\"
            \"target-features\"=\"+cx8,+fxsr,+mmx,+sse,+sse2,+x87\"
            \"tune-cpu\"=\"generic\"
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
