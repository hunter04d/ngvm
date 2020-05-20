use crate::code::Chunk;
use crate::operations::markers::*;
use crate::refs::refs_size;
use crate::types::Type;

use super::process_fallible_bi_op;
use crate::error::VmError;
use crate::interpreter::handlers::alu::AluExtensions;
use crate::operations::{BiOp, BiOpMarker};
use crate::vm::{Vm, VmRefSource};
use std::ops::Try;
use std::option::NoneError;

fn handle_bi_unsigned_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError>
where
    u64: BiOp<M>,
    u32: BiOp<M>,
    u16: BiOp<M>,
    u8: BiOp<M>,
    <u64 as BiOp<M>>::Output: Try<Ok = u64, Error = NoneError>,
    <u32 as BiOp<M>>::Output: Try<Ok = u32, Error = NoneError>,
    <u16 as BiOp<M>>::Output: Try<Ok = u16, Error = NoneError>,
    <u8 as BiOp<M>>::Output: Try<Ok = u8, Error = NoneError>,
{
    let code = chunk.single_opcode();
    let rf = &chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(rf)?;

    if meta.op1.value_type == meta.op2.value_type {
        match meta.op1.value_type {
            Type::U64 => process_fallible_bi_op::<M, u64, u64>(vm, rf, code),
            Type::U32 => process_fallible_bi_op::<M, u32, u32>(vm, rf, code),
            Type::U16 => process_fallible_bi_op::<M, u16, u16>(vm, rf, code),
            Type::U8 => process_fallible_bi_op::<M, u8, u8>(vm, rf, code),
            _ => Err(VmError::InvalidTypeForOperation(
                chunk.single_opcode(),
                meta.op1.value_type,
            )),
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

macro_rules! handle_u_ops {
   ($($fn_name: ident => $marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            handle_bi_unsigned_op::<$marker>(chunk, vm)
        })*

    };
}

handle_u_ops! {
    handle_u_add => CheckedAdd,
    handle_u_sub => CheckedSub,
    handle_u_mul => CheckedMul,
    handle_u_div => CheckedDiv,
    handle_u_rem => CheckedRem
}
