use crate::code::{Chunk, RefSource};
use crate::error::VmError;
use crate::interpreter::handlers::alu::{AluExtensions, TwoStackMetadata};
use crate::opcodes::Opcode;
use crate::operations::markers::*;
use crate::operations::{BiOp, BiOpMarker, UOp, UOpMarker};
use crate::refs::{refs_size, ThreeStackRefs, TwoStackRefs};
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::{HasVmType, Type};
use crate::vm::{Vm, VmRefSource};

fn handle_bi_float_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError>
where
    f64: BiOp<M>,
    f32: BiOp<M>,
    <f64 as BiOp<M>>::Output: HasVmType + IntoStackData,
    <f32 as BiOp<M>>::Output: HasVmType + IntoStackData,
{
    let rf = &chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(rf)?;

    if meta.op1.value_type == meta.op2.value_type {
        match meta.op1.value_type {
            Type::F64 => process_bi_op::<M, f64>(vm, rf, chunk.single_opcode())?,
            Type::F32 => process_bi_op::<M, f32>(vm, rf, chunk.single_opcode())?,
            _ => {
                return Err(VmError::InvalidTypeForOperation(
                    chunk.single_opcode(),
                    meta.op1.value_type,
                ))
            }
        }
    } else {
        return Err(VmError::OperandsTypeMismatch(
            chunk.single_opcode(),
            meta.op1.value_type,
            meta.op2.value_type,
        ));
    }
    Ok(1 + refs_size(3))
}

fn process_bi_op<M, T>(vm: &mut Vm, refs: &ThreeStackRefs, opcode: Opcode) -> Result<(), VmError>
where
    M: BiOpMarker,
    T: BiOp<M> + FromSingle<StackData>,
    <T as BiOp<M>>::Output: IntoStackData + HasVmType,
{
    let meta = vm.three_stack_metadata(refs)?;
    let op1 = T::from_single(*vm.stack_data(meta.op1.index)?);
    let op2 = T::from_single(*vm.stack_data(meta.op2.index)?);
    let r = op1.invoke(op2);
    let res_index = meta.result.index;
    if meta.result.value_type == <T as BiOp<M>>::Output::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return Err(VmError::OutputTypeMismatch(opcode, meta.result.value_type));
    }
    Ok(())
}

fn handle_u_float_op<M: UOpMarker>(chunk: &Chunk, vm: &mut Vm) -> Result<(), VmError>
where
    f64: UOp<M>,
    f32: UOp<M>,
    <f64 as UOp<M>>::Output: HasVmType + IntoStackData,
    <f32 as UOp<M>>::Output: HasVmType + IntoStackData,
{
    let code = chunk.single_opcode();
    let rf = &chunk.read_two().ok_or(VmError::InvalidBytecode)?;

    let meta = vm.two_stack_metadata(rf)?;

    match meta.op.value_type {
        Type::F64 => process_u_op::<M, f64>(vm, rf, code),
        Type::F32 => process_u_op::<M, f64>(vm, rf, code),
        _ => Err(VmError::InvalidTypeForOperation(
            chunk.single_opcode(),
            meta.op.value_type,
        )),
    }
}

fn process_u_op<M, T>(vm: &mut Vm, refs: &TwoStackRefs, opcode: Opcode) -> Result<(), VmError>
where
    M: UOpMarker,
    T: UOp<M> + FromSingle<StackData>,
    <T as UOp<M>>::Output: IntoStackData + HasVmType,
{
    let TwoStackMetadata { result, op } = vm.two_stack_metadata(refs)?;
    let op = T::from_single(*vm.stack_data(op.index)?);
    let r = op.invoke();
    let res_index = result.index;
    if result.value_type == <T as UOp<M>>::Output::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return Err(VmError::OutputTypeMismatch(opcode, result.value_type));
    }
    Ok(())
}

macro_rules! handle_f_ops {
   ($($fn_name: ident => $method_marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError>{
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

pub(in crate::interpreter) fn handle_f_neg(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_u_float_op::<Neg>(chunk, vm)?;
    Ok(1 + refs_size(2))
}
