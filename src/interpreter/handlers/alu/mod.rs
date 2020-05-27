use std::ops::Try;
use std::option::NoneError;

use crate::error::VmError;
use crate::operations::{BiOp, BiOpMarker, UOp, UOpMarker};
use crate::refs::{ThreeStackRefs, TwoStackRefs};
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::stack::metadata::StackMetadata;
use crate::types::checker::{ThreeTypesChecker, TwoTypesChecker, TypeCheckerCtx};
use crate::types::HasPrimitiveType;
use crate::vm::Vm;

pub mod bool_ops;
pub mod cmp_ops;
pub mod f_ops;
pub mod i_ops;
pub mod logic_ops;
pub mod shifts;
pub mod u_ops;

fn process_fallible_bi_op<M: BiOpMarker, T, O>(
    vm: &mut Vm,
    refs: &ThreeStackRefs,
) -> Result<(), VmError>
where
    T: FromSingle<StackData> + BiOp<M, O>,
    O: FromSingle<StackData>,
    <T as BiOp<M, O>>::Output: Try<Error = NoneError>,
    // see: https://github.com/rust-lang/rust/issues/52662
    <<T as BiOp<M, O>>::Output as Try>::Ok: IntoStackData + HasPrimitiveType,
{
    let op1 = T::from_single(*vm.single_stack_data(refs.op1)?);
    let op2 = O::from_single(*vm.single_stack_data(refs.op2)?);
    let r = op1
        .invoke(op2)
        .into_result()
        .map_err(|_| VmError::BiOpError)?;
    *vm.single_stack_data_mut(refs.result)? = r.into_stack_data();
    Ok(())
}

fn process_fallible_u_op<M: UOpMarker, T>(
    vm: &mut Vm,
    TwoStackRefs { result, op }: &TwoStackRefs,
) -> Result<(), VmError>
where
    T: FromSingle<StackData> + UOp<M>,
    <T as UOp<M>>::Output: Try<Error = NoneError>,
    // see: https://github.com/rust-lang/rust/issues/52662
    <<T as UOp<M>>::Output as Try>::Ok: IntoStackData + HasPrimitiveType,
{
    let op = T::from_single(*vm.single_stack_data(*op)?);
    let r = op.invoke().into_result().map_err(|_| VmError::UOpError)?;
    *vm.single_stack_data_mut(*result)? = r.into_stack_data();
    Ok(())
}

fn process_bi_op<M, T>(vm: &mut Vm, refs: &ThreeStackRefs) -> Result<(), VmError>
where
    M: BiOpMarker,
    T: BiOp<M> + FromSingle<StackData>,
    <T as BiOp<M>>::Output: IntoStackData + HasPrimitiveType,
{
    let op1 = T::from_single(*vm.single_stack_data(refs.op1)?);
    let op2 = T::from_single(*vm.single_stack_data(refs.op2)?);
    let r = op1.invoke(op2);
    *vm.single_stack_data_mut(refs.result)? = r.into_stack_data();
    Ok(())
}

fn process_u_op<M, T>(vm: &mut Vm, refs: &TwoStackRefs) -> Result<(), VmError>
where
    M: UOpMarker,
    T: UOp<M> + FromSingle<StackData>,
    <T as UOp<M>>::Output: IntoStackData + HasPrimitiveType,
{
    let op = T::from_single(*vm.single_stack_data(refs.op)?);
    let r = op.invoke();
    *vm.single_stack_data_mut(refs.result)? = r.into_stack_data();
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

impl<'a> ThreeStackMetadata<'a> {
    fn check<'c>(&'a self, ctx: &'c mut TypeCheckerCtx) -> ThreeTypesChecker<'a, 'c> {
        ThreeTypesChecker {
            result: &self.result.value_type,
            op1: &self.op1.value_type,
            op2: &self.op2.value_type,
            ctx,
        }
    }
}

impl<'a> TwoStackMetadata<'a> {
    fn check<'c>(&'a self, ctx: &'c mut TypeCheckerCtx) -> TwoTypesChecker<'a, 'c> {
        TwoTypesChecker {
            result: &self.result.value_type,
            op: &self.op.value_type,
            ctx,
        }
    }
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
