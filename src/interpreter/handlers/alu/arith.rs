//! Arithmetic ops, the operation is picked by the type of the operands:
//! integers use the checked operation, floats use the IEEE one

use crate::code::{Chunk, refs::refs_size};
use crate::error::VmError;
use crate::operations::markers::*;
use crate::operations::{BiOp, BiOpMarker, UOp, UOpMarker};
use crate::stack::data::IntoStackData;
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx};
use crate::types::{HasPrimitiveType, PrimitiveType};
use crate::vm::{Vm, VmRefSource};

use super::{
    AluExtensions, process_bi_op, process_fallible_bi_op, process_fallible_u_op, process_u_op,
};

fn handle_bi_arith_op<I: BiOpMarker, F: BiOpMarker>(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError>
where
    u64: BiOp<I>,
    u32: BiOp<I>,
    u16: BiOp<I>,
    u8: BiOp<I>,
    i64: BiOp<I>,
    i32: BiOp<I>,
    i16: BiOp<I>,
    i8: BiOp<I>,
    f64: BiOp<F>,
    f32: BiOp<F>,
    <u64 as BiOp<I>>::Output: Into<Option<u64>>,
    <u32 as BiOp<I>>::Output: Into<Option<u32>>,
    <u16 as BiOp<I>>::Output: Into<Option<u16>>,
    <u8 as BiOp<I>>::Output: Into<Option<u8>>,
    <i64 as BiOp<I>>::Output: Into<Option<i64>>,
    <i32 as BiOp<I>>::Output: Into<Option<i32>>,
    <i16 as BiOp<I>>::Output: Into<Option<i16>>,
    <i8 as BiOp<I>>::Output: Into<Option<i8>>,
    <f64 as BiOp<F>>::Output: HasPrimitiveType + IntoStackData,
    <f32 as BiOp<F>>::Output: HasPrimitiveType + IntoStackData,
{
    let rf = &chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(rf)?;
    let mut type_checker_ctx = TypeCheckerCtx::new();
    let t = meta
        .check(&mut type_checker_ctx)
        .all_primitives()
        .all_same()
        .number()
        .get_vm()?;
    match t {
        PrimitiveType::U64 => process_fallible_bi_op::<I, u64, u64>(vm, rf),
        PrimitiveType::U32 => process_fallible_bi_op::<I, u32, u32>(vm, rf),
        PrimitiveType::U16 => process_fallible_bi_op::<I, u16, u16>(vm, rf),
        PrimitiveType::U8 => process_fallible_bi_op::<I, u8, u8>(vm, rf),
        PrimitiveType::I64 => process_fallible_bi_op::<I, i64, i64>(vm, rf),
        PrimitiveType::I32 => process_fallible_bi_op::<I, i32, i32>(vm, rf),
        PrimitiveType::I16 => process_fallible_bi_op::<I, i16, i16>(vm, rf),
        PrimitiveType::I8 => process_fallible_bi_op::<I, i8, i8>(vm, rf),
        PrimitiveType::F64 => process_bi_op::<F, f64>(vm, rf),
        PrimitiveType::F32 => process_bi_op::<F, f32>(vm, rf),
        _ => unreachable!("{t:?} passed the number check"),
    }?;
    Ok(1 + refs_size(3))
}

fn handle_u_arith_op<I: UOpMarker, F: UOpMarker>(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError>
where
    i64: UOp<I>,
    i32: UOp<I>,
    i16: UOp<I>,
    i8: UOp<I>,
    f64: UOp<F>,
    f32: UOp<F>,
    <i64 as UOp<I>>::Output: Into<Option<i64>>,
    <i32 as UOp<I>>::Output: Into<Option<i32>>,
    <i16 as UOp<I>>::Output: Into<Option<i16>>,
    <i8 as UOp<I>>::Output: Into<Option<i8>>,
    <f64 as UOp<F>>::Output: HasPrimitiveType + IntoStackData,
    <f32 as UOp<F>>::Output: HasPrimitiveType + IntoStackData,
{
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
        PrimitiveType::I64 => process_fallible_u_op::<I, i64>(vm, rf),
        PrimitiveType::I32 => process_fallible_u_op::<I, i32>(vm, rf),
        PrimitiveType::I16 => process_fallible_u_op::<I, i16>(vm, rf),
        PrimitiveType::I8 => process_fallible_u_op::<I, i8>(vm, rf),
        PrimitiveType::F64 => process_u_op::<F, f64>(vm, rf),
        PrimitiveType::F32 => process_u_op::<F, f32>(vm, rf),
        _ => unreachable!("{t:?} passed the signed or float check"),
    }?;
    Ok(1 + refs_size(2))
}

macro_rules! handle_arith_ops {
   ($($fn_name: ident => $int_marker: ty, $float_marker: ty);* $(;)?) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            handle_bi_arith_op::<$int_marker, $float_marker>(chunk, vm)
        })*
    };
}

handle_arith_ops! {
    handle_add => CheckedAdd, Add;
    handle_sub => CheckedSub, Sub;
    handle_mul => CheckedMul, Mul;
    handle_div => CheckedDiv, Div;
    handle_rem => CheckedRem, Rem;
}

pub(in crate::interpreter) fn handle_neg(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_u_arith_op::<CheckedNeg, Neg>(chunk, vm)
}
