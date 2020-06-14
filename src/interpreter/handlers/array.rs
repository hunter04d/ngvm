//! TODO: refactor

use crate::code::Chunk;
use crate::code::refs::{refs_size, refs_size_with_offset};
use crate::error::VmError;
use crate::meta::{Meta, TransientMeta};
use crate::stack::data::IntoPrimitive;
use crate::types::{HasVmType, RefKind, RefLocation, RefType, VmType};
use crate::types::checker::{combine_checks, HasTypeCheckerCtx};
use crate::vm::{ValueLocation, VmRefSource};
use crate::Vm;
use crate::vm::lock::{ValueLock, ValueLockData};
use crate::vm::refs::LocatedRef;

pub(in crate::interpreter) fn handle_s_arr_create_0(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    let pool = vm.current_const_pool();
    let size = chunk.read_offset_vm()?;
    let type_of = chunk.read_ref_pool_with_offset_vm(0)?;
    let type_of = pool.get_type(type_of).ok_or(VmError::ConstantPoolError)?;
    if !type_of.is_single() {
        return Err(VmError::ConstantPoolError);
    }
    vm.push_array_0(size, type_of);
    Ok(1 + refs_size_with_offset(1))
}

pub(in crate::interpreter) fn handle_s_arr_get(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    let cycle = vm.current_cycle();
    let arr_ref = chunk.read_ref_stack_vm(0)?;
    let index_ref = chunk.read_ref_stack_vm(1)?;

    let arr_ref_meta = vm.stack_metadata(arr_ref)?;
    if arr_ref_meta.cycle >= cycle {
        return Err(VmError::SameCycleRef(RefKind::Mut, arr_ref));
    }
    let index_meta = vm.stack_metadata(index_ref)?;
    let arr_type = arr_ref_meta
        .check("s_arr_ref")
        .any_ref()
        .to()
        .s_arr()
        .and()
        .get();
    let index_type = index_meta.check("index").primitive().unsigned().get();
    let (ref_type, _) = combine_checks(arr_type, index_type).map_err(VmError::TypeError)?;

    let ref_data = vm.single_stack_data(arr_ref)?;
    // get the array location
    let arr_loc_ref = ref_type.locate(ref_data);
    // type of an element of an array
    let (arr_location, ptr) = get_arr_data(vm, arr_loc_ref)?;
    let ptr = ptr.clone();

    vm.stack_metadata_mut(arr_ref)?
        .lock
        .add_mut_lock_partial(cycle)
        .map_err(|e| VmError::LockError(e, ValueLocation::Stack(arr_ref.0)))?;
    let index_value: usize = vm.single_stack_data(index_ref)?.into_primitive();
    let value_location = match arr_location {
        ValueLocation::Stack(si) => ValueLocation::Stack(si + index_value * ptr.size()),
        ValueLocation::Heap(_) => unimplemented!(),
    };
    let meta = TransientMeta {
        value_type: ptr.clone(),
        root_object: arr_loc_ref,
        lock: ValueLock::Ref(ValueLockData {
            lock_cycle: vm.current_cycle(),
            partial_lock: false,
        }),
        was_moved: false,
    };

    vm.transient_refs.insert(value_location, meta);
    let ref_type = RefType {
        kind: RefKind::Ref,
        points_to: RefLocation::TransientOnStack,
        pointer: ptr,
    };
    vm.push_single_typed(value_location, ref_type);
    Ok(1 + refs_size(2))
}

pub(in crate::interpreter) fn handle_s_arr_mut(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    let cycle = vm.current_cycle();
    let arr_ref = chunk.read_ref_stack_vm(0)?;
    let index_ref = chunk.read_ref_stack_vm(1)?;

    let arr_ref_meta = vm.stack_metadata(arr_ref)?;
    if arr_ref_meta.cycle >= cycle {
        return Err(VmError::SameCycleRef(RefKind::Mut, arr_ref));
    }
    let index_meta = vm.stack_metadata(index_ref)?;
    let arr_type = arr_ref_meta
        .check("s_arr_ref")
        .mut_ref()
        .to()
        .s_arr()
        .and()
        .get();
    let index_type = index_meta.check("index").primitive().unsigned().get();
    let (ref_type, _) = combine_checks(arr_type, index_type).map_err(VmError::TypeError)?;

    let ref_data = vm.single_stack_data(arr_ref)?;
    // get the array location
    let arr_loc_ref = ref_type.locate(ref_data);
    // type of an element of an array
    let (arr_location, ptr) = get_arr_data(vm, arr_loc_ref)?;
    let ptr = ptr.clone();
    vm.stack_metadata_mut(arr_ref)?
        .lock
        .add_mut_lock_partial(cycle)
        .map_err(|e| VmError::LockError(e, ValueLocation::Stack(arr_ref.0)))?;

    let index_value: usize = vm.single_stack_data(index_ref)?.into_primitive();
    let value_location = match arr_location {
        ValueLocation::Stack(si) => ValueLocation::Stack(si + index_value * ptr.size()),
        ValueLocation::Heap(_) => unimplemented!(),
    };
    if let Some(t_meta) = vm.transient_refs.get_mut(&value_location) {
        t_meta
            .lock
            .add_mut_lock(cycle)
            .map_err(|e| VmError::LockError(e, arr_location))?;
    } else {
        let meta = TransientMeta {
            value_type: ptr.clone(),
            root_object: arr_loc_ref,
            lock: ValueLock::Mut(ValueLockData {
                lock_cycle: vm.current_cycle(),
                partial_lock: false,
            }),
            was_moved: false,
        };
        vm.transient_refs.insert(value_location, meta);
    }

    let ref_type = RefType {
        kind: RefKind::Mut,
        points_to: RefLocation::TransientOnStack,
        pointer: ptr,
    };
    vm.push_single_typed(value_location, ref_type);
    Ok(1 + refs_size(2))
}

fn get_arr_data(vm: &Vm, located_ref: LocatedRef) -> Result<(ValueLocation, &VmType), VmError> {
    let (loc, arr_type) = match located_ref {
        LocatedRef::Stack(sr) => {
            let stack_meta = vm.stack_metadata(sr)?;
            (ValueLocation::from(stack_meta.index), stack_meta.vm_type())
        }
        LocatedRef::Transient(loc) => {
            let meta = vm.transient_refs.get(&loc).ok_or(VmError::BadVmState)?;
            (loc, meta.vm_type())
        }
    };
    arr_type
        .s_arr()
        .map(|arr| (loc, &arr.pointer))
        .ok_or(VmError::BadVmState)
}
