//! Arithmetic ops, the operation is picked by the type of the operands:
//! integers use the checked operation, floats use the IEEE one

use crate::code::refs::ThreeStackRefs;
use crate::code::{Chunk, refs::refs_size};
use crate::error::VmError;
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx};
use crate::types::{NumberType, SignedOrFloatType};
use crate::vm::{Vm, VmRefSource};

use super::{
    AluExtensions, process_bi_op, process_fallible_bi_op, process_fallible_u_op, process_u_op,
};

/// Reads the operands of a binary arithmetic op and checks they are all the same number type
fn bi_arith_operands(chunk: &Chunk, vm: &Vm) -> Result<(ThreeStackRefs, NumberType), VmError> {
    let rf = chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(&rf)?;
    let mut type_checker_ctx = TypeCheckerCtx::new();
    let t = meta
        .check(&mut type_checker_ctx)
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
                NumberType::U64 => process_fallible_bi_op(vm, rf, u64::$checked),
                NumberType::U32 => process_fallible_bi_op(vm, rf, u32::$checked),
                NumberType::U16 => process_fallible_bi_op(vm, rf, u16::$checked),
                NumberType::U8 => process_fallible_bi_op(vm, rf, u8::$checked),
                NumberType::I64 => process_fallible_bi_op(vm, rf, i64::$checked),
                NumberType::I32 => process_fallible_bi_op(vm, rf, i32::$checked),
                NumberType::I16 => process_fallible_bi_op(vm, rf, i16::$checked),
                NumberType::I8 => process_fallible_bi_op(vm, rf, i8::$checked),
                NumberType::F64 => process_bi_op(vm, rf, |a: f64, b: f64| a $op b),
                NumberType::F32 => process_bi_op(vm, rf, |a: f32, b: f32| a $op b),
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
        .all_same()
        .is::<SignedOrFloatType>()
        .get_vm()?;
    match t {
        SignedOrFloatType::I64 => process_fallible_u_op(vm, rf, i64::checked_neg),
        SignedOrFloatType::I32 => process_fallible_u_op(vm, rf, i32::checked_neg),
        SignedOrFloatType::I16 => process_fallible_u_op(vm, rf, i16::checked_neg),
        SignedOrFloatType::I8 => process_fallible_u_op(vm, rf, i8::checked_neg),
        SignedOrFloatType::F64 => process_u_op(vm, rf, |a: f64| -a),
        SignedOrFloatType::F32 => process_u_op(vm, rf, |a: f32| -a),
    }?;
    Ok(1 + refs_size(2))
}
