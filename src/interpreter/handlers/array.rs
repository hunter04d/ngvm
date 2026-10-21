//! TODO: refactor

use crate::code::Chunk;
use crate::code::refs::{refs_size, refs_size_with_offset};
use crate::error::VmError;
use crate::meta::Meta;
use crate::stack::data::IntoPrimitive;
use crate::types::{HasVmType, RefKind, VmType};
use crate::types::checker::{combine_checks, HasTypeCheckerCtx};
use crate::vm::{Access, ValueLocation, VmRefSource};
use crate::Vm;
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
    if size.checked_mul(type_of.size()).is_none() {
        return Err(VmError::InvalidBytecode);
    }
    vm.push_array_0(size, type_of);
    Ok(1 + refs_size_with_offset(1))
}

pub(in crate::interpreter) fn handle_s_arr_get(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    handle_s_arr_borrow(chunk, vm, RefKind::Ref)
}

pub(in crate::interpreter) fn handle_s_arr_mut(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    handle_s_arr_borrow(chunk, vm, RefKind::Mut)
}

/// Borrows an element of an array through a reference to the array
fn handle_s_arr_borrow(chunk: &Chunk, vm: &mut Vm, kind: RefKind) -> Result<usize, VmError> {
    let cycle = vm.current_cycle();
    let arr_ref = chunk.read_ref_stack_vm(0)?;
    let index_ref = chunk.read_ref_stack_vm(1)?;

    let arr_ref_meta = vm.stack_metadata(arr_ref)?;
    if arr_ref_meta.was_moved {
        return Err(VmError::UseOfMovedValue(arr_ref));
    }
    if arr_ref_meta.cycle >= cycle {
        return Err(VmError::SameCycleRef(kind, arr_ref));
    }
    let index_meta = vm.checked_stack_metadata(index_ref, Access::Read)?;
    let arr_ref_checker = arr_ref_meta.check("s_arr_ref");
    let arr_type = match kind {
        RefKind::Ref => arr_ref_checker.any_ref(),
        RefKind::Mut => arr_ref_checker.mut_ref(),
    }
    .to(|t| t.s_arr())
    .get();
    let index_type = index_meta.check("index").primitive().unsigned().get();
    let (ref_type, _) = combine_checks(arr_type, index_type).map_err(VmError::TypeError)?;

    let arr_loc_ref = ref_type.locate(vm.single_stack_data(arr_ref)?);
    let (arr_location, element_type, len) = get_arr_data(vm, arr_loc_ref)?;
    let index: usize = vm.single_stack_data(index_ref)?.into_primitive();
    if index >= len {
        return Err(VmError::IndexOutOfBounds { index, len });
    }
    let location = match arr_location {
        ValueLocation::Stack(start) => ValueLocation::Stack(start + index * element_type.size()),
        ValueLocation::Heap(_) => unimplemented!(),
    };
    vm.push_element_ref(arr_ref, location, element_type, kind)?;
    Ok(1 + refs_size(2))
}

/// Location, element type and length of the array
fn get_arr_data(vm: &Vm, located_ref: LocatedRef) -> Result<(ValueLocation, VmType, usize), VmError> {
    let (loc, arr_type) = match located_ref {
        LocatedRef::Stack(index) => {
            let stack_meta = vm.absolute_metadata(index)?;
            (ValueLocation::from(stack_meta.index), stack_meta.vm_type())
        }
        LocatedRef::Transient(loc) => {
            let meta = vm.transient_refs.get(&loc).ok_or(VmError::BadVmState)?;
            (loc, meta.vm_type())
        }
    };
    arr_type
        .s_arr()
        .map(|arr| (loc, arr.pointer.clone(), arr.len))
        .ok_or(VmError::BadVmState)
}
