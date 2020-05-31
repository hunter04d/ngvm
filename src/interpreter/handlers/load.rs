use crate::code::{refs::refs_size, Chunk};
use crate::error::VmError;
use crate::stack::data::IntoStackData;
use crate::types::PrimitiveType;
use crate::vm::{Vm, VmRefSource};

pub(in crate::interpreter) fn handle_u64_ld0(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.push_primitive_zeroed(PrimitiveType::U64);
    Ok(1)
}

pub(in crate::interpreter) fn handle_i64_ld0(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.push_primitive_zeroed(PrimitiveType::I64);
    Ok(1)
}

pub(in crate::interpreter) fn handle_ld_unit(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.push_primitive(Default::default(), PrimitiveType::Unit);
    Ok(1)
}

pub(in crate::interpreter) fn handle_ld_typed0(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref_pool_vm(0)?;
    let t = pool.get_type(type_ref).ok_or(VmError::ConstantPoolError)?;
    vm.push_primitive(Default::default(), t);
    Ok(1 + refs_size(1))
}
pub(in crate::interpreter) fn handle_ld_type(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref_pool_vm(0)?;
    let value_ref = chunk.read_ref_pool_vm(1)?;
    let t = pool.get_type(type_ref).ok_or(VmError::ConstantPoolError)?;
    let v = pool
        .get_single(value_ref)
        .ok_or(VmError::ConstantPoolError)?;
    vm.push_primitive(v, t);

    Ok(1 + refs_size(2))
}

pub(in crate::interpreter) fn handle_ld_true(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.push_primitive(true.into_stack_data(), PrimitiveType::Bool);
    Ok(1)
}

pub(in crate::interpreter) fn handle_ld_false(_: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    vm.push_primitive(false.into_stack_data(), PrimitiveType::Bool);
    Ok(1)
}
