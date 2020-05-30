use std::ops::Try;
use std::option::NoneError;

use crate::code::{refs::refs_size, Chunk};
use crate::error::VmError;
use crate::interpreter::handlers::alu::{process_fallible_bi_op, AluExtensions};
use crate::operations::markers::*;
use crate::operations::{BiOp, BiOpMarker};
use crate::types::checker::{HasTypeCheckerCtx, Taggable, TypeCheckerCtx};
use crate::types::{PrimitiveType, VmType};
use crate::vm::{Vm, VmRefSource};

fn handle_shift_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError>
where
    u64: BiOp<M, u32>,
    u32: BiOp<M, u32>,
    u16: BiOp<M, u32>,
    u8: BiOp<M, u32>,
    i64: BiOp<M, u32>,
    i32: BiOp<M, u32>,
    i16: BiOp<M, u32>,
    i8: BiOp<M, u32>,
    <u64 as BiOp<M, u32>>::Output: Try<Ok = u64, Error = NoneError>,
    <u32 as BiOp<M, u32>>::Output: Try<Ok = u32, Error = NoneError>,
    <u16 as BiOp<M, u32>>::Output: Try<Ok = u16, Error = NoneError>,
    <u8 as BiOp<M, u32>>::Output: Try<Ok = u8, Error = NoneError>,
    <i64 as BiOp<M, u32>>::Output: Try<Ok = i64, Error = NoneError>,
    <i32 as BiOp<M, u32>>::Output: Try<Ok = i32, Error = NoneError>,
    <i16 as BiOp<M, u32>>::Output: Try<Ok = i16, Error = NoneError>,
    <i8 as BiOp<M, u32>>::Output: Try<Ok = i8, Error = NoneError>,
{
    let rf = &chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(rf)?;
    let mut type_checker = TypeCheckerCtx::new();
    let types = meta
        .check(&mut type_checker)
        .all_primitives()
        .op2()
        .one_of(&[PrimitiveType::U32, PrimitiveType::U16, PrimitiveType::U8])
        .result()
        .along_with(|c| c.op1())
        .are_same()
        .and()
        .get_vm()?;

    match types.op1 {
        PrimitiveType::U64 => process_fallible_bi_op::<M, u64, u32>(vm, rf),
        PrimitiveType::U32 => process_fallible_bi_op::<M, u32, u32>(vm, rf),
        PrimitiveType::U16 => process_fallible_bi_op::<M, u16, u32>(vm, rf),
        PrimitiveType::U8 => process_fallible_bi_op::<M, u8, u32>(vm, rf),
        PrimitiveType::I64 => process_fallible_bi_op::<M, i64, u32>(vm, rf),
        PrimitiveType::I32 => process_fallible_bi_op::<M, i32, u32>(vm, rf),
        PrimitiveType::I16 => process_fallible_bi_op::<M, i16, u32>(vm, rf),
        PrimitiveType::I8 => process_fallible_bi_op::<M, i8, u32>(vm, rf),
        _ => Err(VmError::InvalidTypeForOperation(
            VmType::from(types.op1).no_tag(),
        )),
    }?;
    Ok(1 + refs_size(3))
}

macro_rules! handle_shifts {
    ($($fn_name: ident => $marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            handle_shift_op::<$marker>(chunk, vm)
        }
        )*
    };
}

handle_shifts! {
    handle_shl => CheckedShl,
    handle_shr => CheckedShr,
    handle_rotl => CheckedRotL,
    handle_rotr => CheckedRotR
}
