use crate::ir::instr::TerminatorInstruction;
use super::*;
use crate::ir::value::{BlockLabel, LocalRegister, Value};
use crate::sema::{ConversionOperation, TypeHandle};

pub fn fmt_negate(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    operand: &Value,
) -> std::fmt::Result {
    let operand_type = operand.get_type();
    let result_llvm = result.llvm(context);
    let type_llvm = operand_type.llvm(context);
    let operand_llvm = operand.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = sub nsw {type_llvm} 0, {operand_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } => {
            write!(f, "{result_llvm} = sub {type_llvm} 0, {operand_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fneg {type_llvm} {operand_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_add(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = add nsw {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } => {
            write!(f, "{result_llvm} = add nuw {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fadd {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_subtract(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = sub nsw {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } => {
            write!(f, "{result_llvm} = sub nuw {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fsub {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_multiply(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = mul nsw {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } => {
            write!(f, "{result_llvm} = mul nuw {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fmul {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_divide(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = sdiv {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } => {
            write!(f, "{result_llvm} = udiv {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fdiv {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_remainder(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = srem {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } => {
            write!(f, "{result_llvm} = urem {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = frem {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_shift_left(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    write!(f, "{result_llvm} = shl {type_llvm} {lhs_llvm}, {rhs_llvm}")
}

pub fn fmt_shift_right(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = ashr {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } => {
            write!(f, "{result_llvm} = lshr {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_not(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    operand: &Value,
) -> std::fmt::Result {
    let operand_type = operand.get_type();
    let result_llvm = result.llvm(context);
    let type_llvm = operand_type.llvm(context);
    let operand_llvm = operand.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Boolean => {
            write!(f, "{result_llvm} = xor {type_llvm} {operand_llvm}, true")
        },
        _ => {
            write!(f, "{result_llvm} = xor {type_llvm} {operand_llvm}, -1")
        }
    }
}

pub fn fmt_and(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    write!(f, "{result_llvm} = and {type_llvm} {lhs_llvm}, {rhs_llvm}")
}

pub fn fmt_or(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    write!(f, "{result_llvm} = or {type_llvm} {lhs_llvm}, {rhs_llvm}")
}

pub fn fmt_xor(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    write!(f, "{result_llvm} = xor {type_llvm} {lhs_llvm}, {rhs_llvm}")
}

pub fn fmt_extract_value(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    aggregate: &Value,
    indices: &[Value],
) -> std::fmt::Result {
    write!(
        f,
        "{} = extractvalue {} {}",
        result.llvm(context),
        aggregate.get_type().llvm(context),
        aggregate.llvm(context),
    )?;

    for index in indices {
        write!(f, ", {} {}", index.get_type().llvm(context), index.llvm(context))?;
    }

    Ok(())
}

pub fn fmt_insert_value(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    aggregate: &Value,
    value: &Value,
    indices: &[Value],
) -> std::fmt::Result {
    write!(
        f,
        "{} = insertvalue {} {}, {} {}",
        result.llvm(context),
        aggregate.get_type().llvm(context),
        aggregate.llvm(context),
        value.get_type().llvm(context),
        value.llvm(context),
    )?;

    for index in indices {
        write!(f, ", {} {}", index.get_type().llvm(context), index.llvm(context))?;
    }

    Ok(())
}

pub fn fmt_stack_allocate(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
) -> std::fmt::Result {
    let &TypeRepr::Pointer { pointee_type, .. } = result.get_type().repr(context) else {
        panic!("{} is not a pointer", result.llvm(context));
    };

    write!(f, "{} = alloca {}", result.llvm(context), pointee_type.llvm(context))
}

pub fn fmt_load(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    pointer: &Value,
) -> std::fmt::Result {
    write!(
        f,
        "{} = load {}, {} {}",
        result.llvm(context),
        result.get_type().llvm(context),
        pointer.get_type().llvm(context),
        pointer.llvm(context),
    )
}

pub fn fmt_store(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    value: &Value,
    pointer: &Value,
) -> std::fmt::Result {
    write!(
        f,
        "store {} {}, {} {}",
        value.get_type().llvm(context),
        value.llvm(context),
        pointer.get_type().llvm(context),
        pointer.llvm(context),
    )
}

pub fn fmt_get_element_pointer(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    pointer: &Value,
    indices: &[Value],
) -> std::fmt::Result {
    let &TypeRepr::Pointer { pointee_type, .. } = pointer.get_type().repr(context) else {
        panic!("{} is not a pointer", pointer.llvm(context));
    };

    write!(
        f,
        "{} = getelementptr inbounds {}, {} {}",
        result.llvm(context),
        pointee_type.llvm(context),
        pointer.get_type().llvm(context),
        pointer.llvm(context),
    )?;

    for index in indices {
        write!(f, ", {} {}", index.get_type().llvm(context), index.llvm(context))?;
    }

    Ok(())
}

pub fn fmt_convert(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    operation: ConversionOperation,
    result: &LocalRegister,
    value: &Value,
) -> std::fmt::Result {
    write!(
        f,
        "{} = {} {} {} to {}",
        result.llvm(context),
        operation.llvm(context),
        value.get_type().llvm(context),
        value.llvm(context),
        result.get_type().llvm(context),
    )
}

pub fn fmt_compare_equal(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { .. } | TypeRepr::Boolean | TypeRepr::Pointer { .. } => {
            write!(f, "{result_llvm} = icmp eq {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fcmp oeq {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_compare_not_equal(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { .. } | TypeRepr::Boolean | TypeRepr::Pointer { .. } => {
            write!(f, "{result_llvm} = icmp ne {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fcmp une {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_compare_less_than(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = icmp slt {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } | TypeRepr::Pointer { .. } => {
            write!(f, "{result_llvm} = icmp ult {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fcmp olt {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_compare_less_equal(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = icmp sle {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } | TypeRepr::Pointer { .. } => {
            write!(f, "{result_llvm} = icmp ule {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fcmp ole {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_compare_greater_than(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = icmp sgt {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } | TypeRepr::Pointer { .. } => {
            write!(f, "{result_llvm} = icmp ugt {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fcmp ogt {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_compare_greater_equal(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    lhs: &Value,
    rhs: &Value,
) -> std::fmt::Result {
    let operand_type = lhs.get_type();
    let type_llvm = operand_type.llvm(context);
    let result_llvm = result.llvm(context);
    let lhs_llvm = lhs.llvm(context);
    let rhs_llvm = rhs.llvm(context);

    match operand_type.repr(context) {
        TypeRepr::Integer { signed: true, .. } => {
            write!(f, "{result_llvm} = icmp sge {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Integer { signed: false, .. } | TypeRepr::Pointer { .. } => {
            write!(f, "{result_llvm} = icmp uge {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        TypeRepr::Float32 | TypeRepr::Float64 => {
            write!(f, "{result_llvm} = fcmp oge {type_llvm} {lhs_llvm}, {rhs_llvm}")
        },
        _ => unimplemented!()
    }
}

pub fn fmt_call(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: Option<&LocalRegister>,
    callee: &Value,
    arguments: &[Value],
) -> std::fmt::Result {
    let TypeRepr::Function { signature } = callee.get_type().repr(context) else {
        panic!("{} is not a function", callee.llvm(context));
    };

    if let Some(result) = result {
        write!(f, "{} = ", result.llvm(context))?;
    }
    write!(f, "call {}(", signature.return_type().llvm(context))?;

    let mut parameters_iter = signature.parameter_types().iter();
    if let Some(&parameter_type) = parameters_iter.next() {
        write!(f, "{}", parameter_type.llvm(context))?;
        for &parameter_type in parameters_iter {
            write!(f, ", {}", parameter_type.llvm(context))?;
        }
        if signature.is_variadic() {
            write!(f, ", ...")?;
        }
    }
    else if signature.is_variadic() {
        write!(f, "...")?;
    }

    write!(f, ") {}(", callee.llvm(context))?;

    let mut arguments_iter = arguments.iter();
    if let Some(argument) = arguments_iter.next() {
        write!(f, "{} {}", argument.get_type().llvm(context), argument.llvm(context))?;
        for argument in arguments_iter {
            write!(f, ", {} {}", argument.get_type().llvm(context), argument.llvm(context))?;
        }
    }

    write!(f, ")")
}

pub fn fmt_phi(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    result: &LocalRegister,
    inputs: &[(Value, BlockLabel)],
) -> std::fmt::Result {
    let mut inputs = inputs.iter();

    let (value, label) = inputs.next()
        .expect("inputs cannot be empty");
    write!(
        f,
        "\t{} = phi {} [ {}, {} ]",
        result.llvm(context),
        result.get_type().llvm(context),
        value.llvm(context),
        label.llvm(context),
    )?;

    for (value, label) in inputs {
        write!(f, ", [ {}, {} ]", value.llvm(context), label.llvm(context))?;
    }

    Ok(())
}

pub fn fmt_return(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    value: &Value,
) -> std::fmt::Result {
    let value_type = value.get_type();

    if value_type == TypeHandle::VOID || value_type == TypeHandle::NEVER {
        write!(f, "ret void")
    }
    else {
        write!(f, "ret {} {}", value_type.llvm(context), value.llvm(context))
    }
}

pub fn fmt_branch(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    to_label: &BlockLabel,
) -> std::fmt::Result {
    write!(f, "br label {}", to_label.llvm(context))
}

pub fn fmt_conditional_branch(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
    condition: &Value,
    consequent_label: &BlockLabel,
    alternative_label: &BlockLabel,
) -> std::fmt::Result {
    write!(
        f,
        "br {} {}, label {}, label {}",
        condition.get_type().llvm(context),
        condition.llvm(context),
        consequent_label.llvm(context),
        alternative_label.llvm(context),
    )
}

pub fn fmt_unreachable(
    f: &mut std::fmt::Formatter<'_>,
    context: &GlobalContext,
) -> std::fmt::Result {
    let _ = context;
    write!(f, "unreachable")
}

impl LLVMDisplay for crate::ir::instr::Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        match self {
            Self::Negate { result, operand } => {
                fmt_negate(f, context, result, operand)
            }
            Self::Add { result, lhs, rhs } => {
                fmt_add(f, context, result, lhs, rhs)
            }
            Self::Subtract { result, lhs, rhs } => {
                fmt_subtract(f, context, result, lhs, rhs)
            }
            Self::Multiply { result, lhs, rhs } => {
                fmt_multiply(f, context, result, lhs, rhs)
            }
            Self::Divide { result, lhs, rhs } => {
                fmt_divide(f, context, result, lhs, rhs)
            }
            Self::Remainder { result, lhs, rhs } => {
                fmt_remainder(f, context, result, lhs, rhs)
            }
            Self::ShiftLeft { result, lhs, rhs } => {
                fmt_shift_left(f, context, result, lhs, rhs)
            }
            Self::ShiftRight { result, lhs, rhs } => {
                fmt_shift_right(f, context, result, lhs, rhs)
            }
            Self::Not { result, operand } => {
                fmt_not(f, context, result, operand)
            }
            Self::And { result, lhs, rhs } => {
                fmt_and(f, context, result, lhs, rhs)
            }
            Self::Or { result, lhs, rhs } => {
                fmt_or(f, context, result, lhs, rhs)
            }
            Self::Xor { result, lhs, rhs } => {
                fmt_xor(f, context, result, lhs, rhs)
            }
            Self::ExtractValue { result, aggregate, indices } => {
                fmt_extract_value(f, context, result, aggregate, indices)
            }
            Self::InsertValue { result, aggregate, value, indices } => {
                fmt_insert_value(f, context, result, aggregate, value, indices)
            }
            Self::StackAllocate { result } => {
                fmt_stack_allocate(f, context, result)
            }
            Self::Load { result, pointer } => {
                fmt_load(f, context, result, pointer)
            }
            Self::Store { value, pointer } => {
                fmt_store(f, context, value, pointer)
            }
            Self::GetElementPointer { result, pointer, indices } => {
                fmt_get_element_pointer(f, context, result, pointer, indices)
            }
            Self::Convert { operation, result, value } => {
                fmt_convert(f, context, *operation, result, value)
            }
            Self::CompareEqual { result, lhs, rhs } => {
                fmt_compare_equal(f, context, result, lhs, rhs)
            }
            Self::CompareNotEqual { result, lhs, rhs } => {
                fmt_compare_not_equal(f, context, result, lhs, rhs)
            }
            Self::CompareLessThan { result, lhs, rhs } => {
                fmt_compare_less_than(f, context, result, lhs, rhs)
            }
            Self::CompareLessEqual { result, lhs, rhs } => {
                fmt_compare_less_equal(f, context, result, lhs, rhs)
            }
            Self::CompareGreaterThan { result, lhs, rhs } => {
                fmt_compare_greater_than(f, context, result, lhs, rhs)
            }
            Self::CompareGreaterEqual { result, lhs, rhs } => {
                fmt_compare_greater_equal(f, context, result, lhs, rhs)
            }
            Self::Call { result, callee, arguments } => {
                fmt_call(f, context, result.as_ref(), callee, arguments)
            }
        }
    }
}

impl LLVMDisplay for crate::ir::instr::PhiInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        fmt_phi(f, context, &self.result, &self.inputs)
    }
}

impl LLVMDisplay for crate::ir::instr::TerminatorInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, context: &GlobalContext) -> std::fmt::Result {
        match self {
            TerminatorInstruction::Return { value } => {
                fmt_return(f, context, value)
            }
            TerminatorInstruction::Branch { to_label } => {
                fmt_branch(f, context, to_label)
            }
            TerminatorInstruction::ConditionalBranch { condition, consequent_label, alternative_label } => {
                fmt_conditional_branch(f, context, condition, consequent_label, alternative_label)
            }
            TerminatorInstruction::Unreachable => {
                fmt_unreachable(f, context)
            }
        }
    }
}
