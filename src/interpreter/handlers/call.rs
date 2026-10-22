use crate::code::{refs::refs_size, Chunk};
use crate::error::VmError;
use crate::vm::VmRefSource;
use crate::Vm;

pub(in crate::interpreter) fn handle_call(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let f = chunk.read_ref_pool_vm(0)?;
    let return_ip = vm.ip + 1 + refs_size(1);
    vm.call(f, return_ip)?;
    Ok(0)
}

pub(in crate::interpreter) fn handle_ret(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let value = chunk.read_ref_stack_vm(0)?;
    vm.ret(value)?;
    Ok(0)
}

pub(in crate::interpreter) fn handle_ret_n(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let count = chunk.read_offset_vm()?;
    if count == 0 {
        // a function returns at least one value
        return Err(VmError::InvalidBytecode);
    }
    let first = chunk.read_ref_stack_with_offset_vm(0)?;
    vm.ret_values(first, count)?;
    Ok(0)
}
