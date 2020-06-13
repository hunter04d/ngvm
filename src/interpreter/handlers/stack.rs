use smallvec::SmallVec;

use crate::code::refs::refs_size;
use crate::code::Chunk;
use crate::error::VmError;
use crate::stack::data::StackData;
use crate::types::checker::{Taggable, TypeError};
use crate::vm::VmRefSource;
use crate::Vm;

pub(in crate::interpreter) fn handle_mv(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let result = chunk.read_ref_stack_vm(0)?;
    let op = chunk.read_ref_stack_vm(1)?;
    let result_meta = vm.stack_metadata(result)?;
    let op_meta = vm.stack_metadata(op)?;

    let from = result_meta.index.0;
    let until = from + result_meta.value_type.size();
    if result_meta.value_type != op_meta.value_type {
        let result_type = result_meta.value_type.clone().tag("r");
        let op_type = op_meta.value_type.clone().tag("o");
        let e = TypeError::TwoNotEqual(result_type, op_type);
        return Err(VmError::TypeError(vec![e]));
    }
    if result_meta.value_type.ref_type().is_some() {
            let op_type = op_meta.value_type.clone().tag("o");
            let msg = "Cannot move a reference. NOTE: references are not designed to be moved right now, create a new reference instead";
            let e = TypeError::Condition(op_type, msg.into());
            return Err(VmError::TypeError(vec![e]));
    }

    vm.free_by_index(result)?;
    let op_meta = vm.stack_metadata_mut(op)?;
    if op_meta.was_moved {
        return Err(VmError::UseOfMovedValue(op));
    }
    if !op_meta.value_type.is_copy() {
        op_meta.was_moved = true;
    }

    let value = vm
        .stack_data(op)?
        .iter()
        .copied()
        .collect::<SmallVec<[StackData; 2]>>();

    vm.stack.splice(from..until, value);
    vm.stack_metadata_mut(result)?.was_moved = false;
    Ok(1 + refs_size(2))
}
