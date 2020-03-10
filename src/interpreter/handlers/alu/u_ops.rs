use crate::code::Chunk;
use crate::interpreter::{run, three_stack_metadata, InterpreterResult};
use crate::operations::markers::*;
use crate::refs::refs;
use crate::types::Type;
use crate::Vm;

use super::process_fallible_bi_op;
use crate::error::VmError;
use crate::error::VmError::InvalidBytecode;
use crate::operations::{BiOp, BiOpMarker};
use std::ops::Try;
use std::option::NoneError;

fn handle_bi_unsigned_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult
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
    let result = run(|| {
        let rf = &chunk.read_three().ok_or(InvalidBytecode)?;

        let meta = three_stack_metadata(vm, rf)?;

        if meta.op1.value_type == meta.op2.value_type {
            match meta.op1.value_type {
                Type::U64 => process_fallible_bi_op::<M, u64, u64>(vm, rf),
                Type::U32 => process_fallible_bi_op::<M, u32, u32>(vm, rf),
                Type::U16 => process_fallible_bi_op::<M, u16, u16>(vm, rf),
                Type::U8 => process_fallible_bi_op::<M, u8, u8>(vm, rf),
                _ => Err(VmError::InvalidTypeForOperation(
                    chunk.opcode(),
                    meta.op1.value_type,
                )),
            }
        } else {
            Err(VmError::OperandsTypeMismatch(
                chunk.opcode(),
                meta.op1.value_type,
                meta.op2.value_type,
            ))
        }
    });
    InterpreterResult::new(1 + refs(3)).with_error_opt(result.err())
}

macro_rules! handle_u_ops {
   ($($fn_name: ident => $marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult {
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
