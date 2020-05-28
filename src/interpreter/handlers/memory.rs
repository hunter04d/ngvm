use crate::code::Chunk;
use crate::error::VmError;
use crate::refs::refs_size;
use crate::types::RefKind;
use crate::vm::VmRefSource;
use crate::Vm;

pub(in crate::interpreter) fn handle_start_scope(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.push_scope()?;
    Ok(1)
}

pub(in crate::interpreter) fn handle_end_scope(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let current_cycle = vm.current_cycle();

    while let Some(meta) = vm.stack_metadata.last() {
        if meta.cycle < current_cycle {
            break;
        }
        vm.pop_stack();
    }
    vm.pop_scope()?;
    Ok(1)
}

fn handle_take_lock(chunk: &Chunk, vm: &mut Vm, kind: RefKind) -> Result<usize, VmError> {
    let rf = chunk.read_ref_stack_vm(0)?;
    let meta = vm.stack_metadata(rf)?;
    if vm.cycle <= meta.cycle {
        return Err(VmError::SameCycleRef(kind, rf));
    }
    vm.push_stack_ref(rf, kind)?;

    Ok(1 + refs_size(1))
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
