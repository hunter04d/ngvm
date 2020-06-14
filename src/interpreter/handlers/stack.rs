use smallvec::SmallVec;

use crate::{vm, Vm};
use crate::code::Chunk;
use crate::code::refs::{refs_size, StackRef};
use crate::error::VmError;
use crate::stack::data::StackData;
use crate::types::{RefKind, RefType, VmType};
use crate::types::checker::{Taggable, TypeError};
use crate::vm::VmRefSource;

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
    if let (Some(result_r), Some(op_r)) = (
        result_meta.value_type.ref_type(),
        op_meta.value_type.ref_type(),
    ) {
        check_ref_move_rules(vm, result, result_r)?;
        check_ref_move_rules(vm, op, op_r)?;
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
    vm.stack_metadata_mut(op)?.was_moved = false;
    Ok(1 + refs_size(2))
}

pub(in crate::interpreter) fn handle_mp(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let op = chunk.read_ref_stack_vm(0)?;
    let op_meta = vm.stack_metadata(op)?;

    if let Some(r) = op_meta.value_type.ref_type() {
        check_ref_move_rules(vm, op, r)?;
    }
    let op_meta = vm.stack_metadata_mut(op)?;
    if op_meta.was_moved {
        return Err(VmError::UseOfMovedValue(op));
    }
    if !op_meta.value_type.is_copy() {
        op_meta.was_moved = true;
    }
    let t = op_meta.value_type.clone();
    let value = vm
        .stack_data(op)?
        .iter()
        .copied()
        .collect::<SmallVec<[StackData; 2]>>();

    vm.push_typed(value, t);
    vm.stack_metadata_mut(op)?.was_moved = false;
    Ok(1 + refs_size(2))
}

fn check_ref_move_rules(vm: &Vm, op: StackRef, r: &RefType) -> vm::Result<()> {
    let cycle = vm.current_cycle();
    if matches!(r.kind, RefKind::Ref) {
        let ref_value = vm.single_stack_data(op)?;
        let located_ref = r.locate(ref_value);
        let value_meta = vm.meta_view(located_ref)?;
        if let Some(c) = value_meta.lock().lock_cycle() {
            if c <= cycle {
                let op_type = VmType::from(r.clone()).tag("o");
                let msg = "Cannot move a root ref reference. Create a new reference instead";
                let e = TypeError::Condition(op_type, msg.into());
                return Err(VmError::TypeError(vec![e]));
            }
        }
    }
    Ok(())
}
