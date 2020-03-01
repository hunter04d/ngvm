use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use crate::code::Chunk;
use crate::interpreter::{
    three_stack_metadata, two_stack_metadata, TwoStackMetadata,
};
use crate::refs::refs;
use crate::stack::data::StackData;
use crate::stack::{data::FromSingle, data::IntoStackData};
use crate::types::{HasVmType, Type};
use crate::Vm;
use crate::refs::{TwoRefs, ThreeRefs};

pub(crate) struct BiFloatCtx<R64, R32, Fn64, Fn32>
where
    R64: IntoStackData + HasVmType,
    R32: IntoStackData + HasVmType,
    Fn64: Fn(f64, f64) -> R64,
    Fn32: Fn(f32, f32) -> R32,
{
    fn64: Fn64,
    fn32: Fn32,
}

pub(crate) struct UFloatCtx<R64, R32, Fn64, Fn32>
where
    R64: IntoStackData + HasVmType,
    R32: IntoStackData + HasVmType,
    Fn64: Fn(f64) -> R64,
    Fn32: Fn(f32) -> R32,
{
    fn64: Fn64,
    fn32: Fn32,
}

fn handle_bi_float_op<R64, R32, Fn64, Fn32>(
    chunk: &Chunk,
    vm: &mut Vm,
    ctx: BiFloatCtx<R64, R32, Fn64, Fn32>,
) -> usize
where
    R64: IntoStackData + HasVmType,
    R32: IntoStackData + HasVmType,
    Fn64: Fn(f64, f64) -> R64,
    Fn32: Fn(f32, f32) -> R32,
{
    let rf = &chunk.read_three().expect("Invalid Bytecode");

    let meta = three_stack_metadata(vm, rf).expect("Bad vm state");

    if meta.op1.value_type == meta.op2.value_type {
        match meta.op1.value_type {
            Type::F64 => process_bi_op(vm, rf, ctx.fn64).expect("Calc Error"),
            Type::F32 => process_bi_op(vm, rf, ctx.fn32).expect("Calc Error"),
            _ => unreachable!("Types mismatch"),
        }
    } else {
        panic!("Type mismatch");
    }
    1 + refs(3)
}

fn process_bi_op<T, R, Func>(vm: &mut Vm, refs: &ThreeRefs, func: Func) -> Option<()>
where
    T: FromSingle<StackData>,
    R: IntoStackData + HasVmType,
    Func: Fn(T, T) -> R,
{
    let meta = three_stack_metadata(vm, refs)?;
    let op1 = T::from_single(*vm.stack_data(meta.op1.index)?);
    let op2 = T::from_single(*vm.stack_data(meta.op2.index)?);
    let r = func(op1, op2);
    let res_index = meta.result.index;
    if meta.result.value_type == R::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return None;
    }
    Some(())
}

fn handle_u_float_op<R64, R32, Fn64, Fn32>(
    chunk: &Chunk,
    vm: &mut Vm,
    ctx: UFloatCtx<R64, R32, Fn64, Fn32>,
) -> usize
where
    R64: IntoStackData + HasVmType,
    R32: IntoStackData + HasVmType,
    Fn64: Fn(f64) -> R64,
    Fn32: Fn(f32) -> R32,
{
    let rf = &chunk.read_two().expect("Invalid Bytecode");

    let meta = two_stack_metadata(vm, rf).expect("Bad vm state");

    match meta.op.value_type {
        Type::F64 => process_u_op(vm, rf, ctx.fn64).expect("Calc Error"),
        Type::F32 => process_u_op(vm, rf, ctx.fn32).expect("Calc Error"),
        _ => unreachable!("Types mismatch"),
    }
    1 + refs(2)
}

fn process_u_op<T, R, Func>(
    vm: &mut Vm,
    refs: &TwoRefs,
    func: Func,
) -> Option<()>
where
    T: FromSingle<StackData>,
    R: IntoStackData + HasVmType,
    Func: Fn(T) -> R,
{
    let TwoStackMetadata {result, op} = two_stack_metadata(vm, refs)?;
    let op = T::from_single(*vm.stack_data(op.index)?);
    let r = func(op);
    let res_index = result.index;
    if result.value_type == R::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return None;
    }
    Some(())
}

macro_rules! bi_float_ctx_from_method {
    ($method_name: ident) => {
        BiFloatCtx {
            fn64: |v1, v2| v1.$method_name(v2),
            fn32: |v1, v2| v1.$method_name(v2),
        }
    };
}

macro_rules! handle_f_ops {
   ($($fn_name: ident => $method_name: ident),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_bi_float_op(chunk, vm, bi_float_ctx_from_method!($method_name))
        })*

    };
}

handle_f_ops! {
    handle_f_add => add,
    handle_f_sub => sub,
    handle_f_mul => mul,
    handle_f_div => div,
    handle_f_rem => rem
}

pub(in crate::interpreter) fn handle_f_neg(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_u_float_op(
        chunk,
        vm,
        UFloatCtx {
            fn64: |v| v.neg(),
            fn32: |v| v.neg(),
        },
    )
}
