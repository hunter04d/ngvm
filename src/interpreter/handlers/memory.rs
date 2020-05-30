use crate::code::refs::StackRef;
use crate::code::{refs::refs_size, Chunk};
use crate::error::VmError;
use crate::types::RefKind;
use crate::vm::lock::DerefLock;
use crate::vm::refs::LocatedRef;
use crate::vm::{ValueLocation, VmRefSource};
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
        vm.pop_stack()?;
    }
    vm.pop_scope()?;
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
    let cycle = vm.current_cycle();
    let (located_ref, r) = vm.locate_ref(rf)?;
    let r_kind = r.kind;
    let meta = vm.stack_metadata_mut(rf)?;

    if matches!(r_kind, RefKind::Mut) {
        meta.lock
            .add_lock(cycle, RefKind::Mut)
            .map_err(|e| VmError::LockError(e, rf))?;
    }
    match located_ref {
        LocatedRef::Stack(index) => {
            let t = vm.stack_metadata(StackRef(index))?.value_type.clone();
            let v = vm.stack_data(StackRef(index))?.to_vec();
            vm.push_deref(&v, t, r_kind, rf)
        }
        LocatedRef::Transient(index) => {
            let meta = vm.transient_refs.get(&index).ok_or(VmError::BadVmState)?;
            match index {
                ValueLocation::Stack(index) => {
                    let v = vm.stack_data(StackRef(index))?.to_vec();
                    let t = meta.value_type.clone();
                    vm.push_deref(&v, t, r_kind, rf)
                }
                ValueLocation::Heap(_) => unimplemented!(),
            }
        }
    }
    Ok(1 + refs_size(1))
}

pub(in crate::interpreter) fn handle_end_deref(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.pop_deref()?;
    Ok(1)
}
