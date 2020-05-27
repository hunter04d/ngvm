use super::AluExtensions;
use crate::code::Chunk;
use crate::error::VmError;
use crate::refs::refs_size;
use crate::stack::data::{FromPrimitive, FromSingle, IntoStackData, StackData};
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx};
use crate::types::PrimitiveType;
use crate::vm::VmRefSource;
use crate::Vm;

fn handle_bi_op(
    chunk: &Chunk,
    vm: &mut Vm,
    processor: impl FnOnce(StackData, StackData) -> StackData,
) -> Result<usize, VmError> {
    let rf = &chunk.read_three_vm()?;
    let meta = vm.three_stack_metadata(rf)?;
    let mut t_ctx = TypeCheckerCtx::new();
    let _ = meta
        .check(&mut t_ctx)
        .all_primitives()
        .result()
        .equals(PrimitiveType::Bool)
        .operands()
        .same()
        .and()
        .user()
        .get_vm()?;

    let op1 = *vm.single_stack_data(rf.op1)?;
    let op2 = *vm.single_stack_data(rf.op2)?;
    *vm.single_stack_data_mut(rf.result)? = processor(op1, op2);
    Ok(1 + refs_size(3))
}

fn handle_u_op(
    chunk: &Chunk,
    vm: &mut Vm,
    processor: impl Fn(StackData) -> StackData,
) -> Result<usize, VmError> {
    let rf = &chunk.read_two_vm()?;

    let meta = vm.two_stack_metadata(rf)?;

    let t_ctx = &mut TypeCheckerCtx::new();
    let _ = meta
        .check(t_ctx)
        .all_primitives()
        .result()
        .bool()
        .op()
        .user()
        .get_vm()?;

    let op = vm.single_stack_data(rf.op)?;
    *vm.single_stack_data_mut(rf.result)? = processor(*op);
    Ok(1 + refs_size(3))
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
