use crate::code::refs::{ThreeStackRefs, TwoStackRefs};
use crate::error::VmError;
use crate::meta::StackMeta;
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::checker::{OperandTypesChecker, TypeCheckerCtx};
use crate::vm::{Access, Vm};

pub mod arith;
pub mod bool_ops;
pub mod cmp_ops;
pub mod logic_ops;
pub mod shifts;

#[cfg(test)]
mod tests;

fn process_bi_op<A, B, R>(
    vm: &mut Vm,
    refs: &ThreeStackRefs,
    op: impl FnOnce(A, B) -> R,
) -> Result<(), VmError>
where
    A: FromSingle<StackData>,
    B: FromSingle<StackData>,
    R: IntoStackData,
{
    let op1 = A::from_single(*vm.single_stack_data(refs.op1)?);
    let op2 = B::from_single(*vm.single_stack_data(refs.op2)?);
    vm.set_single_stack_data(refs.result, op(op1, op2).into_stack_data())?;
    Ok(())
}

fn process_fallible_bi_op<A, B, R>(
    vm: &mut Vm,
    refs: &ThreeStackRefs,
    op: impl FnOnce(A, B) -> Option<R>,
) -> Result<(), VmError>
where
    A: FromSingle<StackData>,
    B: FromSingle<StackData>,
    R: IntoStackData,
{
    let op1 = A::from_single(*vm.single_stack_data(refs.op1)?);
    let op2 = B::from_single(*vm.single_stack_data(refs.op2)?);
    let r = op(op1, op2).ok_or(VmError::BiOpError)?;
    vm.set_single_stack_data(refs.result, r.into_stack_data())?;
    Ok(())
}

fn process_u_op<A, R>(
    vm: &mut Vm,
    refs: &TwoStackRefs,
    op: impl FnOnce(A) -> R,
) -> Result<(), VmError>
where
    A: FromSingle<StackData>,
    R: IntoStackData,
{
    let a = A::from_single(*vm.single_stack_data(refs.op)?);
    vm.set_single_stack_data(refs.result, op(a).into_stack_data())?;
    Ok(())
}

fn process_fallible_u_op<A, R>(
    vm: &mut Vm,
    refs: &TwoStackRefs,
    op: impl FnOnce(A) -> Option<R>,
) -> Result<(), VmError>
where
    A: FromSingle<StackData>,
    R: IntoStackData,
{
    let a = A::from_single(*vm.single_stack_data(refs.op)?);
    let r = op(a).ok_or(VmError::UOpError)?;
    vm.set_single_stack_data(refs.result, r.into_stack_data())?;
    Ok(())
}

struct ThreeStackMetadata<'a> {
    result: &'a StackMeta,
    op1: &'a StackMeta,
    op2: &'a StackMeta,
}

struct TwoStackMetadata<'a> {
    result: &'a StackMeta,
    op: &'a StackMeta,
}

impl<'a> ThreeStackMetadata<'a> {
    fn check<'c>(&'a self, ctx: &'c mut TypeCheckerCtx) -> OperandTypesChecker<'a, 'c, 3> {
        OperandTypesChecker::three(
            &self.result.value_type,
            &self.op1.value_type,
            &self.op2.value_type,
            ctx,
        )
    }
}

impl<'a> TwoStackMetadata<'a> {
    fn check<'c>(&'a self, ctx: &'c mut TypeCheckerCtx) -> OperandTypesChecker<'a, 'c, 2> {
        OperandTypesChecker::two(&self.result.value_type, &self.op.value_type, ctx)
    }
}

trait AluExtensions {
    fn three_stack_metadata(
        &self,
        refs: &ThreeStackRefs,
    ) -> Result<ThreeStackMetadata<'_>, VmError>;

    fn two_stack_metadata(&self, refs: &TwoStackRefs) -> Result<TwoStackMetadata<'_>, VmError>;
}

impl AluExtensions for Vm {
    fn three_stack_metadata(
        &self,
        refs: &ThreeStackRefs,
    ) -> Result<ThreeStackMetadata<'_>, VmError> {
        let result = self.checked_stack_metadata(refs.result, Access::Write)?;
        let op1 = self.checked_stack_metadata(refs.op1, Access::Read)?;
        let op2 = self.checked_stack_metadata(refs.op2, Access::Read)?;
        Ok(ThreeStackMetadata { result, op1, op2 })
    }

    fn two_stack_metadata(&self, refs: &TwoStackRefs) -> Result<TwoStackMetadata<'_>, VmError> {
        let result = self.checked_stack_metadata(refs.result, Access::Write)?;
        let op = self.checked_stack_metadata(refs.op, Access::Read)?;
        Ok(TwoStackMetadata { result, op })
    }
}
