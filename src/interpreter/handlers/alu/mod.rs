use crate::code::Chunk;
use crate::error::VmError;
use crate::interpreter::{three_stack_metadata, two_stack_metadata, InterpreterResult};
use crate::operations::{BiOp, BiOpMarker, UOp, UOpMarker};
use crate::refs;
use crate::refs::{ThreeRefs, TwoRefs};
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::{HasVmType, Type};
use crate::Vm;
use std::ops::Try;
use std::option::NoneError;

pub mod f_ops;
pub mod i_ops;
pub mod shifts;
pub mod u_ops;

fn process_fallible_bi_op<M: BiOpMarker, T, O>(vm: &mut Vm, refs: &ThreeRefs) -> Result<(), VmError>
where
    T: FromSingle<StackData> + BiOp<M, O>,
    O: FromSingle<StackData>,
    <T as BiOp<M, O>>::Output: Try<Error = NoneError>,
    // see: https://github.com/rust-lang/rust/issues/52662
    <<T as BiOp<M, O>>::Output as Try>::Ok: IntoStackData + HasVmType,
{
    let meta = three_stack_metadata(vm, refs)?;
    let op1 = T::from_single(*vm.stack_data(meta.op1.index)?);
    let op2 = O::from_single(*vm.stack_data(meta.op2.index)?);
    let r = op1
        .invoke(op2)
        .into_result()
        .map_err(|_| VmError::BiOpError)?;
    let res_index = meta.result.index;
    if meta.result.value_type == <<T as BiOp<M, O>>::Output as Try>::Ok::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return Err(VmError::OutputTypeMismatch);
    }
    Ok(())
}

fn process_fallible_u_op<M: UOpMarker, T>(
    vm: &mut Vm,
    TwoRefs { result, op }: &TwoRefs,
) -> Result<(), VmError>
where
    T: FromSingle<StackData> + UOp<M>,
    <T as UOp<M>>::Output: Try<Error = NoneError>,
    // see: https://github.com/rust-lang/rust/issues/52662
    <<T as UOp<M>>::Output as Try>::Ok: IntoStackData + HasVmType,
{
    let op = vm.stack_metadata(*op)?;
    let result = vm.stack_metadata(*result)?;
    let op = T::from_single(*vm.stack_data(op.index)?);
    let r = op.invoke().into_result().map_err(|_| VmError::UOpError)?;
    let res_index = result.index;
    if result.value_type == <<T as UOp<M>>::Output as Try>::Ok::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return Err(VmError::OutputTypeMismatch);
    }
    Ok(())
}

pub(in crate::interpreter) fn handle_b_not(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult {
    let refs = chunk.read_two().expect("Ok");
    let meta = two_stack_metadata(vm, &refs).unwrap();
    if meta.result.value_type != Type::Bool {
        panic!("Types mismatch")
    }
    if !meta.op.value_type.is_primitive() {
        panic!("Invalid operation")
    }

    let data = *vm.stack_data(meta.op.index).unwrap();
    let res_index = meta.result.index;
    *vm.stack_data_mut(res_index).unwrap() = data.iter().any(|&v| v != 0u8).into_stack_data();
    InterpreterResult::new(1 + refs::refs(2))
}
