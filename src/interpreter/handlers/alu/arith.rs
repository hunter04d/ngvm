//! Arithmetic ops, the operation is picked by the type of the operands:
//! integers use the checked operation, floats use the IEEE one

use crate::code::refs::ThreeStackRefs;
use crate::code::{Chunk, refs::refs_size};
use crate::error::VmError;
use crate::types::PrimitiveType;
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx};
use crate::vm::{Vm, VmRefSource};

use super::{
    AluExtensions, process_bi_op, process_fallible_bi_op, process_fallible_u_op, process_u_op,
};

/// Reads the operands of a binary arithmetic op and checks they are all the same number type
fn bi_arith_operands(chunk: &Chunk, vm: &Vm) -> Result<(ThreeStackRefs, PrimitiveType), VmError> {
    let rf = chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(&rf)?;
    let mut type_checker_ctx = TypeCheckerCtx::new();
    let t = meta
        .check(&mut type_checker_ctx)
        .all_primitives()
        .all_same()
        .number()
        .get_vm()?;
    Ok((rf, t))
}

macro_rules! handle_arith_ops {
    ($($fn_name: ident => $checked: ident, $op: tt);* $(;)?) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            let (rf, t) = &bi_arith_operands(chunk, vm)?;
            match t {
                PrimitiveType::U64 => process_fallible_bi_op(vm, rf, u64::$checked),
                PrimitiveType::U32 => process_fallible_bi_op(vm, rf, u32::$checked),
                PrimitiveType::U16 => process_fallible_bi_op(vm, rf, u16::$checked),
                PrimitiveType::U8 => process_fallible_bi_op(vm, rf, u8::$checked),
                PrimitiveType::I64 => process_fallible_bi_op(vm, rf, i64::$checked),
                PrimitiveType::I32 => process_fallible_bi_op(vm, rf, i32::$checked),
                PrimitiveType::I16 => process_fallible_bi_op(vm, rf, i16::$checked),
                PrimitiveType::I8 => process_fallible_bi_op(vm, rf, i8::$checked),
                PrimitiveType::F64 => process_bi_op(vm, rf, |a: f64, b: f64| a $op b),
                PrimitiveType::F32 => process_bi_op(vm, rf, |a: f32, b: f32| a $op b),
                _ => unreachable!("{t:?} passed the number check"),
            }?;
            Ok(1 + refs_size(3))
        }
        )*
    };
}

handle_arith_ops! {
    handle_add => checked_add, +;
    handle_sub => checked_sub, -;
    handle_mul => checked_mul, *;
    handle_div => checked_div, /;
    handle_rem => checked_rem, %;
}

pub(in crate::interpreter) fn handle_neg(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let rf = &chunk.read_two_vm()?;

    let meta = vm.two_stack_metadata(rf)?;
    let t_ctx = &mut TypeCheckerCtx::new();
    let t = meta
        .check(t_ctx)
        .all_primitives()
        .all_same()
        .either()
        .signed()
        .or()
        .float()
        .and()
        .fmt(|_, t| {
            format!(
                "<{}>{:?} is neither signed integer or float",
                t.tag, t.vm_type
            )
        })
        .get_vm()?
        .op;
    match t {
        PrimitiveType::I64 => process_fallible_u_op(vm, rf, i64::checked_neg),
        PrimitiveType::I32 => process_fallible_u_op(vm, rf, i32::checked_neg),
        PrimitiveType::I16 => process_fallible_u_op(vm, rf, i16::checked_neg),
        PrimitiveType::I8 => process_fallible_u_op(vm, rf, i8::checked_neg),
        PrimitiveType::F64 => process_u_op(vm, rf, |a: f64| -a),
        PrimitiveType::F32 => process_u_op(vm, rf, |a: f32| -a),
        _ => unreachable!("{t:?} passed the signed or float check"),
    }?;
    Ok(1 + refs_size(2))
}
