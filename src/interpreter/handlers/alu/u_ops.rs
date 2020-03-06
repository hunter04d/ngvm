use crate::code::Chunk;
use crate::interpreter::three_stack_metadata;
use crate::operations::markers::*;
use crate::refs::refs;
use crate::types::Type;
use crate::Vm;

use super::process_fallible_bi_op;
use crate::operations::{BiOp, BiOpMarker};
use std::ops::Try;
use std::option::NoneError;

fn handle_bi_unsigned_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> usize
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
    let rf = &chunk.read_three().expect("Invalid Bytecode");

    let meta = three_stack_metadata(vm, rf).expect("Bad vm state");

    if meta.op1.value_type == meta.op2.value_type {
        match meta.op1.value_type {
            Type::U64 => process_fallible_bi_op::<M, u64, u64>(vm, rf).expect("Calc Error"),
            Type::U32 => process_fallible_bi_op::<M, u32, u32>(vm, rf).expect("Calc Error"),
            Type::U16 => process_fallible_bi_op::<M, u16, u16>(vm, rf).expect("Calc Error"),
            Type::U8 => process_fallible_bi_op::<M, u8, u8>(vm, rf).expect("Calc Error"),
            _ => unreachable!("Types mismatch"),
        }
    } else {
        panic!("Type mismatch");
    }
    1 + refs(3)
}

macro_rules! handle_u_ops {
   ($($fn_name: ident => $marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
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
