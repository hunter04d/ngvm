use crate::code::{refs::refs_size, Chunk};
use crate::error::VmError;
use crate::types::RefKind;
use crate::vm::lock::DerefLock;
use crate::vm::VmRefSource;
use crate::Vm;

pub(in crate::interpreter) fn handle_start_scope(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.push_scope()?;
    Ok(1)
}

pub(in crate::interpreter) fn handle_end_scope(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.end_scope()?;
    Ok(1)
}

fn handle_take_lock(chunk: &Chunk, vm: &mut Vm, kind: RefKind) -> Result<usize, VmError> {
    let rf = chunk.read_ref_stack_vm(0)?;
    let meta = vm.stack_metadata(rf)?;
    if meta.deref != DerefLock::None {
        Err(VmError::RefToTemp(kind, rf))
    } else if vm.cycle <= meta.cycle {
        Err(VmError::SameCycleRef(kind, rf))
    } else {
        vm.push_stack_ref(rf, kind)?;
        Ok(1 + refs_size(1))
    }
}
pub(in crate::interpreter) fn handle_take_ref(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    handle_take_lock(chunk, vm, RefKind::Ref)
}

pub(in crate::interpreter) fn handle_take_mut(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    handle_take_lock(chunk, vm, RefKind::Mut)
}

pub(in crate::interpreter) fn handle_start_deref(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    let rf = chunk.read_ref_stack_vm(0)?;
    vm.push_deref(rf)?;
    Ok(1 + refs_size(1))
}

pub(in crate::interpreter) fn handle_end_deref(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.pop_deref()?;
    Ok(1)
}
