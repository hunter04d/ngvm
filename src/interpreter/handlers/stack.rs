use crate::Vm;
use crate::code::Chunk;
use crate::code::refs::refs_size;
use crate::error::VmError;
use crate::types::checker::{Taggable, TypeError};
use crate::vm::VmRefSource;

pub(in crate::interpreter) fn handle_mv(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let result = chunk.read_ref_stack_vm(0)?;
    let op = chunk.read_ref_stack_vm(1)?;
    let result_meta = vm.stack_metadata(result)?;
    let op_meta = vm.stack_metadata(op)?;

    if result_meta.value_type != op_meta.value_type {
        let result_type = result_meta.value_type.clone().tag("r");
        let op_type = op_meta.value_type.clone().tag("o");
        let e = TypeError::TwoNotEqual(result_type, op_type);
        return Err(VmError::TypeError(vec![e]));
    }
    vm.move_into(result, op)?;
    Ok(1 + refs_size(2))
}

pub(in crate::interpreter) fn handle_mp(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let op = chunk.read_ref_stack_vm(0)?;
    vm.push_moved(op)?;
    Ok(1 + refs_size(1))
}
