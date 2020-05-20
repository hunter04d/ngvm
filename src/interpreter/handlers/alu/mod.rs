use std::ops::Try;
use std::option::NoneError;

use crate::error::VmError;
use crate::opcodes::Opcode;
use crate::operations::{BiOp, BiOpMarker, UOp, UOpMarker};
use crate::refs::{ThreeStackRefs, TwoStackRefs};
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::stack::metadata::StackMetadata;
use crate::types::HasVmType;
use crate::vm::Vm;

pub mod bool_ops;
pub mod f_ops;
pub mod i_ops;
pub mod shifts;
pub mod u_ops;

fn process_fallible_bi_op<M: BiOpMarker, T, O>(
    vm: &mut Vm,
    refs: &ThreeStackRefs,
    opcode: Opcode,
) -> Result<(), VmError>
where
    T: FromSingle<StackData> + BiOp<M, O>,
    O: FromSingle<StackData>,
    <T as BiOp<M, O>>::Output: Try<Error = NoneError>,
    // see: https://github.com/rust-lang/rust/issues/52662
    <<T as BiOp<M, O>>::Output as Try>::Ok: IntoStackData + HasVmType,
{
    let meta = vm.three_stack_metadata(refs)?;
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
        return Err(VmError::OutputTypeMismatch(opcode, meta.result.value_type));
    }
    Ok(())
}

fn process_fallible_u_op<M: UOpMarker, T>(
    vm: &mut Vm,
    TwoStackRefs { result, op }: &TwoStackRefs,
    opcode: Opcode,
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
        return Err(VmError::OutputTypeMismatch(opcode, result.value_type));
    }
    Ok(())
}

struct ThreeStackMetadata<'a> {
    result: &'a StackMetadata,
    op1: &'a StackMetadata,
    op2: &'a StackMetadata,
}

struct TwoStackMetadata<'a> {
    result: &'a StackMetadata,
    op: &'a StackMetadata,
}

trait AluExtensions {
    fn three_stack_metadata(&self, refs: &ThreeStackRefs) -> Result<ThreeStackMetadata, VmError>;

    fn two_stack_metadata(&self, refs: &TwoStackRefs) -> Result<TwoStackMetadata, VmError>;
}

impl AluExtensions for Vm {
    fn three_stack_metadata(&self, refs: &ThreeStackRefs) -> Result<ThreeStackMetadata, VmError> {
        let result = self.stack_metadata(refs.result)?;
        let op1 = self.stack_metadata(refs.op1)?;
        let op2 = self.stack_metadata(refs.op2)?;
        Ok(ThreeStackMetadata { result, op1, op2 })
    }

    fn two_stack_metadata(&self, refs: &TwoStackRefs) -> Result<TwoStackMetadata, VmError> {
        let result = self.stack_metadata(refs.result)?;
        let op = self.stack_metadata(refs.op)?;
        Ok(TwoStackMetadata { result, op })
    }
}
