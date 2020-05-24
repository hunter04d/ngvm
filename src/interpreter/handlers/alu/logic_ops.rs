use crate::code::Chunk;
use crate::error::VmError;
use crate::refs::refs_size;
use crate::stack::data::StackData;
use crate::types::checker::TypeCheckerCtx;
use crate::Vm;
use crate::vm::VmRefSource;

use super::AluExtensions;

#[allow(dead_code)]
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
        .all_same()
        .and()
        .integer();

    let op1 = *vm.stack_data(meta.op1.index)?;
    let op2 = *vm.stack_data(meta.op2.index)?;
    let index = meta.result.index;
    *vm.stack_data_mut(index)? = processor(op1, op2);
    Ok(1 + refs_size(3))
}

#[allow(dead_code)]
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
        .all_same()
        .and()
        .integer();

    let op = vm.stack_data(meta.op.index)?;
    let index = meta.result.index;
    *vm.stack_data_mut(index)? = processor(*op);
    Ok(1 + refs_size(3))
}
