use crate::code::refs::refs_size_with_offset;
use crate::code::Chunk;
use crate::error::VmError;
use crate::vm::VmRefSource;
use crate::Vm;

pub(in crate::interpreter) fn handle_s_arr_create_0(
    chunk: &Chunk,
    vm: &mut Vm,
) -> Result<usize, VmError> {
    let pool = vm.current_const_pool();
    let size = chunk.read_offset_vm()?;
    let type_of = chunk.read_ref_pool_with_offset_vm(0)?;
    let type_of = pool.get_type(type_of).ok_or(VmError::ConstantPoolError)?;

    vm.push_array_0(size, type_of);
    Ok(1 + refs_size_with_offset(1))
}
