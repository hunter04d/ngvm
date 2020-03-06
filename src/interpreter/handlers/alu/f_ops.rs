use crate::code::Chunk;
use crate::interpreter::{three_stack_metadata, two_stack_metadata, TwoStackMetadata};
use crate::operations::markers::*;
use crate::operations::{BiOp, BiOpMarker, UOp, UOpMarker};
use crate::refs::{refs, ThreeRefs, TwoRefs};
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::{HasVmType, Type};
use crate::Vm;

fn handle_bi_float_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> usize
where
    f64: BiOp<M>,
    f32: BiOp<M>,
    <f64 as BiOp<M>>::Output: HasVmType + IntoStackData,
    <f32 as BiOp<M>>::Output: HasVmType + IntoStackData,
{
    let rf = &chunk.read_three().expect("Invalid Bytecode");

    let meta = three_stack_metadata(vm, rf).expect("Bad vm state");

    if meta.op1.value_type == meta.op2.value_type {
        match meta.op1.value_type {
            Type::F64 => process_bi_op::<M, f64>(vm, rf).expect("Calc Error"),
            Type::F32 => process_bi_op::<M, f32>(vm, rf).expect("Calc Error"),
            _ => unreachable!("Types mismatch"),
        }
    } else {
        panic!("Type mismatch");
    }
    1 + refs(3)
}

fn process_bi_op<M, T>(vm: &mut Vm, refs: &ThreeRefs) -> Option<()>
where
    M: BiOpMarker,
    T: BiOp<M> + FromSingle<StackData>,
    <T as BiOp<M>>::Output: IntoStackData + HasVmType,
{
    let meta = three_stack_metadata(vm, refs)?;
    let op1 = T::from_single(*vm.stack_data(meta.op1.index)?);
    let op2 = T::from_single(*vm.stack_data(meta.op2.index)?);
    let r = op1.invoke(op2);
    let res_index = meta.result.index;
    if meta.result.value_type == <T as BiOp<M>>::Output::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return None;
    }
    Some(())
}

fn handle_u_float_op<M: UOpMarker>(chunk: &Chunk, vm: &mut Vm) -> usize
where
    f64: UOp<M>,
    f32: UOp<M>,
    <f64 as UOp<M>>::Output: HasVmType + IntoStackData,
    <f32 as UOp<M>>::Output: HasVmType + IntoStackData,
{
    let rf = &chunk.read_two().expect("Invalid Bytecode");

    let meta = two_stack_metadata(vm, rf).expect("Bad vm state");

    match meta.op.value_type {
        Type::F64 => process_u_op::<M, f64>(vm, rf).expect("Calc Error"),
        Type::F32 => process_u_op::<M, f64>(vm, rf).expect("Calc Error"),
        _ => unreachable!("Types mismatch"),
    }
    1 + refs(2)
}

fn process_u_op<M, T>(vm: &mut Vm, refs: &TwoRefs) -> Option<()>
where
    M: UOpMarker,
    T: UOp<M> + FromSingle<StackData>,
    <T as UOp<M>>::Output: IntoStackData + HasVmType,
{
    let TwoStackMetadata { result, op } = two_stack_metadata(vm, refs)?;
    let op = T::from_single(*vm.stack_data(op.index)?);
    let r = op.invoke();
    let res_index = result.index;
    if result.value_type == <T as UOp<M>>::Output::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return None;
    }
    Some(())
}

macro_rules! handle_f_ops {
   ($($fn_name: ident => $method_marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_bi_float_op::<$method_marker>(chunk, vm)
        })*

    };
}

handle_f_ops! {
    handle_f_add => Add,
    handle_f_sub => Sub,
    handle_f_mul => Mul,
    handle_f_div => Div,
    handle_f_rem => Rem
}

pub(in crate::interpreter) fn handle_f_neg(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_u_float_op::<Neg>(chunk, vm)
}
