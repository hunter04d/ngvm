use crate::code::Chunk;
use crate::error::VmError;
use crate::interpreter::{
    run, three_stack_metadata, two_stack_metadata, InterpreterResult, TwoStackMetadata,
};
use crate::operations::markers::*;
use crate::operations::{BiOp, BiOpMarker, UOp, UOpMarker};
use crate::refs::{refs, ThreeRefs, TwoRefs};
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::{HasVmType, Type};
use crate::Vm;

fn handle_bi_float_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult
where
    f64: BiOp<M>,
    f32: BiOp<M>,
    <f64 as BiOp<M>>::Output: HasVmType + IntoStackData,
    <f32 as BiOp<M>>::Output: HasVmType + IntoStackData,
{
    let result = run(|| {
        let rf = &chunk.read_three().ok_or(VmError::InvalidBytecode)?;

        let meta = three_stack_metadata(vm, rf)?;

        if meta.op1.value_type == meta.op2.value_type {
            match meta.op1.value_type {
                Type::F64 => process_bi_op::<M, f64>(vm, rf),
                Type::F32 => process_bi_op::<M, f32>(vm, rf),
                _ => Err(VmError::InvalidTypeForOperation(
                    chunk.opcode(),
                    meta.op1.value_type,
                )),
            }
        } else {
            Err(VmError::OperandsTypeMismatch(
                chunk.opcode(),
                meta.op1.value_type,
                meta.op2.value_type,
            ))
        }
    });
    InterpreterResult::new(1 + refs(3)).with_error_opt(result.err())
}

fn process_bi_op<M, T>(vm: &mut Vm, refs: &ThreeRefs) -> Result<(), VmError>
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
        return Err(VmError::OutputTypeMismatch);
    }
    Ok(())
}

fn handle_u_float_op<M: UOpMarker>(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult
where
    f64: UOp<M>,
    f32: UOp<M>,
    <f64 as UOp<M>>::Output: HasVmType + IntoStackData,
    <f32 as UOp<M>>::Output: HasVmType + IntoStackData,
{
    let result = run(|| {
        let rf = &chunk.read_two().ok_or(VmError::InvalidBytecode)?;

        let meta = two_stack_metadata(vm, rf)?;

        match meta.op.value_type {
            Type::F64 => process_u_op::<M, f64>(vm, rf),
            Type::F32 => process_u_op::<M, f64>(vm, rf),
            _ => Err(VmError::InvalidTypeForOperation(
                chunk.opcode(),
                meta.op.value_type,
            )),
        }
    });
    InterpreterResult::new(1 + refs(2)).with_error_opt(result.err())
}

fn process_u_op<M, T>(vm: &mut Vm, refs: &TwoRefs) -> Result<(), VmError>
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
        return Err(VmError::OutputTypeMismatch);
    }
    Ok(())
}

macro_rules! handle_f_ops {
   ($($fn_name: ident => $method_marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult {
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

pub(in crate::interpreter) fn handle_f_neg(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult {
    handle_u_float_op::<Neg>(chunk, vm)
}
