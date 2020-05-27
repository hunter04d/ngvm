use crate::code::Chunk;
use crate::error::VmError;
use crate::refs::refs_size;
use crate::stack::data::IntoPrimitive;
use crate::types::checker::{tags, HasTypeCheckerCtx, TypeCheckerCtx};
use crate::vm::VmRefSource;
use crate::Vm;
use std::mem::size_of;

pub(in crate::interpreter) fn handle_j(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let offset = chunk.read_offset_vm()?;
    vm.ip = offset;
    Ok(0)
}

pub(in crate::interpreter) fn handle_jc(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let offset = chunk.read_offset_vm()?;
    let cond = chunk.read_ref_stack_with_offset_vm(0)?;
    let meta = vm.stack_metadata(cond)?;
    let mut t_ctx = TypeCheckerCtx::new();
    let _ = meta
        .check(tags::COND, &mut t_ctx)
        .primitive()
        .bool()
        .get_vm()?;
    if vm.single_stack_data(cond)?.into_primitive() {
        vm.ip = offset;
        Ok(0)
    } else {
        Ok(1 + size_of::<usize>() + refs_size(1))
    }
}
