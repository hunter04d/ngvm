use std::ops::Try;
use std::option::NoneError;

use crate::code::Chunk;
use crate::interpreter::handlers::alu::process_fallible_bi_op;
use crate::interpreter::{three_stack_metadata};
use crate::operations::markers::*;
use crate::operations::{BiOp, BiOpMarker};
use crate::refs::refs_size;
use crate::types::Type;
use crate::vm::{Vm, VmRefSource};
use crate::error::VmError;

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

    let meta = three_stack_metadata(vm, rf)?;

    if matches!(meta.op2.value_type, Type::U32 | Type::U16 | Type::U8) {
        match meta.op1.value_type {
            Type::U64 => process_fallible_bi_op::<M, u64, u32>(vm, rf),
            Type::U32 => process_fallible_bi_op::<M, u32, u32>(vm, rf),
            Type::U16 => process_fallible_bi_op::<M, u16, u32>(vm, rf),
            Type::U8 => process_fallible_bi_op::<M, u8, u32>(vm, rf),
            Type::I64 => process_fallible_bi_op::<M, i64, u32>(vm, rf),
            Type::I32 => process_fallible_bi_op::<M, i32, u32>(vm, rf),
            Type::I16 => process_fallible_bi_op::<M, i16, u32>(vm, rf),
            Type::I8 => process_fallible_bi_op::<M, i8, u32>(vm, rf),
            _ => Err(VmError::InvalidTypeForOperation(chunk.single_opcode(), meta.op1.value_type)),
        }
    } else {
        Err(VmError::OperandsTypeMismatch(
            chunk.single_opcode(),
            meta.op1.value_type,
            meta.op2.value_type,
        ))
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
