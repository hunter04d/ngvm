use super::{process_bi_op, process_u_op};
use crate::code::Chunk;
use crate::interpreter::{
    three_stack_metadata, two_stack_metadata
};
use crate::refs::refs;
use crate::stack::data::IntoStackData;
use crate::types::{HasVmType, Type};
use crate::Vm;

struct BiUnsignedCtx<R8, R16, R32, R64, Fn8, Fn16, Fn32, Fn64>
where
    Fn8: Fn(u8, u8) -> R8,
    Fn16: Fn(u16, u16) -> R16,
    Fn32: Fn(u32, u32) -> R32,
    Fn64: Fn(u64, u64) -> R64,
{
    fn8: Fn8,
    fn16: Fn16,
    fn32: Fn32,
    fn64: Fn64,
}

#[allow(dead_code)]
struct UUnsignedCtx<R8, R16, R32, R64, Fn8, Fn16, Fn32, Fn64>
where
    Fn8: Fn(u8) -> R8,
    Fn16: Fn(u16) -> R16,
    Fn32: Fn(u32) -> R32,
    Fn64: Fn(u64) -> R64,
{
    fn8: Fn8,
    fn16: Fn16,
    fn32: Fn32,
    fn64: Fn64,
}

fn handle_bi_unsigned_op<R8, R16, R32, R64, Fn8, Fn16, Fn32, Fn64>(
    chunk: &Chunk,
    vm: &mut Vm,
    ctx: BiUnsignedCtx<Option<R8>, Option<R16>, Option<R32>, Option<R64>, Fn8, Fn16, Fn32, Fn64>,
) -> usize
where
    R8: IntoStackData + HasVmType,
    R16: IntoStackData + HasVmType,
    R32: IntoStackData + HasVmType,
    R64: IntoStackData + HasVmType,
    Fn8: Fn(u8, u8) -> Option<R8>,
    Fn16: Fn(u16, u16) -> Option<R16>,
    Fn32: Fn(u32, u32) -> Option<R32>,
    Fn64: Fn(u64, u64) -> Option<R64>,
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

#[allow(dead_code)]
fn handle_u_signed_op<R8, R16, R32, R64, Fn8, Fn16, Fn32, Fn64>(
    chunk: &Chunk,
    vm: &mut Vm,
    ctx: UUnsignedCtx<Option<R8>, Option<R16>, Option<R32>, Option<R64>, Fn8, Fn16, Fn32, Fn64>,
) -> usize
where
    R8: IntoStackData + HasVmType,
    R16: IntoStackData + HasVmType,
    R32: IntoStackData + HasVmType,
    R64: IntoStackData + HasVmType,
    Fn8: Fn(u8) -> Option<R8>,
    Fn16: Fn(u16) -> Option<R16>,
    Fn32: Fn(u32) -> Option<R32>,
    Fn64: Fn(u64) -> Option<R64>,
{
    let rf = &chunk.read_two().expect("Invalid Bytecode");
    let meta = two_stack_metadata(vm, rf).expect("Bad vm state");
    match meta.op.value_type {
        Type::U64 => process_u_op(vm, rf, ctx.fn64).expect("Calc Error"),
        Type::U32 => process_u_op(vm, rf, ctx.fn32).expect("Calc Error"),
        Type::U16 => process_u_op(vm, rf, ctx.fn16).expect("Calc Error"),
        Type::U8 => process_u_op(vm, rf, ctx.fn8).expect("Calc Error"),
        _ => unreachable!("Types mismatch"),
    }
    1 + refs(2)
}

macro_rules! bi_unsigned_ctx_from_method {
    ($method: ident) => {
        BiUnsignedCtx {
            fn8: |v1, v2| v1.$method(v2),
            fn16: |v1, v2| v1.$method(v2),
            fn32: |v1, v2| v1.$method(v2),
            fn64: |v1, v2| v1.$method(v2),
        }
    };
}

macro_rules! u_unsigned_ctx_from_method {
    ($method: ident) => {
        UUnsignedCtx {
            fn8: |v| v.$method(),
            fn16: |v| v.$method(),
            fn32: |v| v.$method(),
            fn64: |v| v.$method(),
        }
    };
}

macro_rules! handle_u_ops {
   ($($fn_name: ident => $method_name: ident),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_bi_unsigned_op(chunk, vm, bi_unsigned_ctx_from_method!($method_name))
        })*

    };
}

handle_u_ops! {
    handle_u_add => checked_add,
    handle_u_sub => checked_sub,
    handle_u_mul => checked_mul,
    handle_u_div => checked_div,
    handle_u_rem => checked_rem
}
