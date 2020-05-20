use super::AluExtensions;
use crate::code::Chunk;
use crate::error::VmError;
use crate::refs::refs_size;
use crate::stack::data::{FromPrimitive, FromSingle, IntoStackData, StackData};
use crate::vm::VmRefSource;
use crate::Vm;

fn handle_bi_op(
    chunk: &Chunk,
    vm: &mut Vm,
    processor: impl FnOnce(StackData, StackData) -> StackData,
) -> Result<usize, VmError> {
    let rf = &chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(rf)?;

    if meta.op1.value_type.is_primitive() && meta.op1.value_type == meta.op2.value_type {
        if meta.result.value_type.is_bool() {
            let op1 = *vm.stack_data(meta.op1.index)?;
            let op2 = *vm.stack_data(meta.op2.index)?;
            let index = meta.result.index;
            *vm.stack_data_mut(index)? = processor(op1, op2);
            Ok(1 + refs_size(3))
        } else {
            Err(VmError::InvalidTypeForOperation(
                chunk.single_opcode(),
                meta.result.value_type,
            ))
        }
    } else {
        Err(VmError::OperandsTypeMismatch(
            chunk.single_opcode(),
            meta.op1.value_type,
            meta.op2.value_type,
        ))
    }
}

fn handle_u_op(
    chunk: &Chunk,
    vm: &mut Vm,
    processor: impl Fn(StackData) -> StackData,
) -> Result<usize, VmError> {
    let rf = &chunk.read_two_vm()?;

    let meta = vm.two_stack_metadata(rf)?;

    if meta.op.value_type.is_primitive() {
        if meta.result.value_type.is_bool() {
            let op = vm.stack_data(meta.op.index)?;
            let index = meta.result.index;
            *vm.stack_data_mut(index)? = processor(*op);
            Ok(1 + refs_size(3))
        } else {
            Err(VmError::OutputTypeMismatch(
                chunk.single_opcode(),
                meta.result.value_type,
            ))
        }
    } else {
        Err(VmError::InvalidTypeForOperation(
            chunk.single_opcode(),
            meta.op.value_type,
        ))
    }
}

fn be(op: StackData) -> bool {
    bool::from_single(op)
}

fn not(op: StackData) -> bool {
    !be(op)
}

fn bi_as_bool(op1: StackData, op2: StackData, h: impl Fn(bool, bool) -> bool) -> StackData {
    StackData::from_primitive(h(be(op1), be(op2)))
}

fn handle_b_bi_op(
    chunk: &Chunk,
    vm: &mut Vm,
    h: impl Fn(bool, bool) -> bool,
) -> Result<usize, VmError> {
    handle_bi_op(chunk, vm, |op1, op2| bi_as_bool(op1, op2, h))
}

pub(in crate::interpreter) fn handle_b_and(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_b_bi_op(chunk, vm, |op1, op2| op1 && op2)
}

pub(in crate::interpreter) fn handle_b_or(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_b_bi_op(chunk, vm, |op1, op2| op1 || op2)
}

pub(in crate::interpreter) fn handle_b_xor(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_b_bi_op(chunk, vm, |op1, op2| op1 ^ op2)
}

pub(in crate::interpreter) fn handle_b_be(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_u_op(chunk, vm, |data| be(data).into_stack_data())
}

pub(in crate::interpreter) fn handle_b_not(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_u_op(chunk, vm, |data| not(data).into_stack_data())
}
