use crate::code::Chunk;
use crate::interpreter::{three_stack_metadata, two_stack_metadata};
use crate::operations::markers::*;
use crate::operations::{BiOp, BiOpMarker, UOp, UOpMarker};
use crate::refs::refs;
use crate::types::Type;
use crate::Vm;

use super::{process_fallible_bi_op, process_fallible_u_op};
use std::ops::Try;
use std::option::NoneError;

fn handle_bi_signed_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> usize
where
    i64: BiOp<M>,
    i32: BiOp<M>,
    i16: BiOp<M>,
    i8: BiOp<M>,
    <i64 as BiOp<M>>::Output: Try<Ok = i64, Error = NoneError>,
    <i32 as BiOp<M>>::Output: Try<Ok = i32, Error = NoneError>,
    <i16 as BiOp<M>>::Output: Try<Ok = i16, Error = NoneError>,
    <i8 as BiOp<M>>::Output: Try<Ok = i8, Error = NoneError>,
{
    let rf = &chunk.read_three().expect("Invalid Bytecode");

    let meta = three_stack_metadata(vm, rf).expect("Bad vm state");

    if meta.op1.value_type == meta.op2.value_type {
        match meta.op1.value_type {
            Type::I64 => process_fallible_bi_op::<M, i64, i64>(vm, rf).expect("Calc Error"),
            Type::I32 => process_fallible_bi_op::<M, i32, i32>(vm, rf).expect("Calc Error"),
            Type::I16 => process_fallible_bi_op::<M, i16, i16>(vm, rf).expect("Calc Error"),
            Type::I8 => process_fallible_bi_op::<M, i8, i8>(vm, rf).expect("Calc Error"),
            _ => unreachable!("Types mismatch"),
        }
    } else {
        panic!("Type mismatch");
    }
    1 + refs(3)
}

fn handle_u_signed_op<M: UOpMarker>(chunk: &Chunk, vm: &mut Vm) -> usize
where
    i64: UOp<M>,
    i32: UOp<M>,
    i16: UOp<M>,
    i8: UOp<M>,
    <i64 as UOp<M>>::Output: Try<Ok = i64, Error = NoneError>,
    <i32 as UOp<M>>::Output: Try<Ok = i32, Error = NoneError>,
    <i16 as UOp<M>>::Output: Try<Ok = i16, Error = NoneError>,
    <i8 as UOp<M>>::Output: Try<Ok = i8, Error = NoneError>,
{
    let rf = &chunk.read_two().expect("Invalid Bytecode");
    let meta = two_stack_metadata(vm, rf).expect("Bad vm state");
    match meta.op.value_type {
        Type::I64 => process_fallible_u_op::<M, i64>(vm, rf),
        Type::I32 => process_fallible_u_op::<M, i32>(vm, rf),
        Type::I16 => process_fallible_u_op::<M, i16>(vm, rf),
        Type::I8 => process_fallible_u_op::<M, i8>(vm, rf),
        _ => None,
    }
    .expect("Bad op");
    1 + refs(2)
}

macro_rules! handle_i_ops {
   ($($fn_name: ident => $marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_bi_signed_op::<$marker>(chunk, vm)
        })*

    };
}

handle_i_ops! {
    handle_i_add => CheckedAdd,
    handle_i_sub => CheckedSub,
    handle_i_mul => CheckedMul,
    handle_i_div => CheckedDiv,
    handle_i_rem => CheckedRem
}

pub(in crate::interpreter) fn handle_i_neg(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_u_signed_op::<CheckedNeg>(chunk, vm)
}
