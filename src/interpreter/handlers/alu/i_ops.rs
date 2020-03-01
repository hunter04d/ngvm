use super::{process_bi_op, process_u_op};
use crate::code::Chunk;
use crate::interpreter::{
    three_stack_metadata, two_stack_metadata
};
use crate::refs::refs;
use crate::stack::data::{ IntoStackData};
use crate::types::{HasVmType, Type};
use crate::Vm;

struct BiSignedCtx<R8, R16, R32, R64, Fn8, Fn16, Fn32, Fn64>
where
    Fn8: Fn(i8, i8) -> R8,
    Fn16: Fn(i16, i16) -> R16,
    Fn32: Fn(i32, i32) -> R32,
    Fn64: Fn(i64, i64) -> R64,
{
    fn8: Fn8,
    fn16: Fn16,
    fn32: Fn32,
    fn64: Fn64,
}

struct USignedCtx<R8, R16, R32, R64, Fn8, Fn16, Fn32, Fn64>
where
    Fn8: Fn(i8) -> R8,
    Fn16: Fn(i16) -> R16,
    Fn32: Fn(i32) -> R32,
    Fn64: Fn(i64) -> R64,
{
    fn8: Fn8,
    fn16: Fn16,
    fn32: Fn32,
    fn64: Fn64,
}

fn handle_bi_signed_op<R8, R16, R32, R64, Fn8, Fn16, Fn32, Fn64>(
    chunk: &Chunk,
    vm: &mut Vm,
    ctx: BiSignedCtx<Option<R8>, Option<R16>, Option<R32>, Option<R64>, Fn8, Fn16, Fn32, Fn64>,
) -> usize
where
    R8: IntoStackData + HasVmType,
    R16: IntoStackData + HasVmType,
    R32: IntoStackData + HasVmType,
    R64: IntoStackData + HasVmType,
    Fn8: Fn(i8, i8) -> Option<R8>,
    Fn16: Fn(i16, i16) -> Option<R16>,
    Fn32: Fn(i32, i32) -> Option<R32>,
    Fn64: Fn(i64, i64) -> Option<R64>,
{
    let rf = &chunk.read_three().expect("Invalid Bytecode");

    let meta = three_stack_metadata(vm, rf).expect("Bad vm state");

    if meta.op1.value_type == meta.op2.value_type {
        match meta.op1.value_type {
            Type::I64 => process_bi_op(vm, rf, ctx.fn64).expect("Calc Error"),
            Type::I32 => process_bi_op(vm, rf, ctx.fn32).expect("Calc Error"),
            Type::I16 => process_bi_op(vm, rf, ctx.fn16).expect("Calc Error"),
            Type::I8 => process_bi_op(vm, rf, ctx.fn8).expect("Calc Error"),
            _ => unreachable!("Types mismatch"),
        }
    } else {
        panic!("Type mismatch");
    }
    1 + refs(3)
}

fn handle_u_signed_op<R8, R16, R32, R64, Fn8, Fn16, Fn32, Fn64>(
    chunk: &Chunk,
    vm: &mut Vm,
    ctx: USignedCtx<Option<R8>, Option<R16>, Option<R32>, Option<R64>, Fn8, Fn16, Fn32, Fn64>,
) -> usize
where
    R8: IntoStackData + HasVmType,
    R16: IntoStackData + HasVmType,
    R32: IntoStackData + HasVmType,
    R64: IntoStackData + HasVmType,
    Fn8: Fn(i8) -> Option<R8>,
    Fn16: Fn(i16) -> Option<R16>,
    Fn32: Fn(i32) -> Option<R32>,
    Fn64: Fn(i64) -> Option<R64>,
{
    let rf = &chunk.read_two().expect("Invalid Bytecode");
    let meta = two_stack_metadata(vm, rf).expect("Bad vm state");
    match meta.op.value_type {
        Type::I64 => process_u_op(vm, rf, ctx.fn64).expect("Calc Error"),
        Type::I32 => process_u_op(vm, rf, ctx.fn32).expect("Calc Error"),
        Type::I16 => process_u_op(vm, rf, ctx.fn16).expect("Calc Error"),
        Type::I8 => process_u_op(vm, rf, ctx.fn8).expect("Calc Error"),
        _ => unreachable!("Types mismatch"),
    }
    1 + refs(2)
}

macro_rules! bi_signed_ctx_from_method {
    ($method: ident) => {
        BiSignedCtx {
            fn8: |v1, v2| v1.$method(v2),
            fn16: |v1, v2| v1.$method(v2),
            fn32: |v1, v2| v1.$method(v2),
            fn64: |v1, v2| v1.$method(v2),
        }
    };
}

macro_rules! handle_i_ops {
   ($($fn_name: ident => $method_name: ident),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_bi_signed_op(chunk, vm, bi_signed_ctx_from_method!($method_name))
        })*

    };
}

handle_i_ops! {
    handle_i_add => checked_add,
    handle_i_sub => checked_sub,
    handle_i_mul => checked_mul,
    handle_i_div => checked_div,
    handle_i_rem => checked_rem
}

macro_rules! u_signed_ctx_from_method {
    ($method: ident) => {
        USignedCtx {
            fn8: |v| v.$method(),
            fn16: |v| v.$method(),
            fn32: |v| v.$method(),
            fn64: |v| v.$method(),
        }
    };
}

pub(in crate::interpreter) fn handle_i_neg(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_u_signed_op(chunk, vm, u_signed_ctx_from_method!(checked_neg))
}
