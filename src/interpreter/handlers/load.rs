use crate::code::Chunk;
use crate::opcodes::refs;
use crate::types::Type;
use crate::{StackValue, Vm};

pub(in crate::interpreter) fn handle_u64_ld0(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(StackValue::default_with_type(Type::U64));
    1
}

pub(in crate::interpreter) fn handle_i64_ld0(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(StackValue::default_with_type(Type::I64));
    1
}

pub(in crate::interpreter) fn handle_ld_unit(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(StackValue::default_with_type(Type::Unit));
    1
}

pub(in crate::interpreter) fn handle_ld_typed0(chunk: &Chunk, vm: &mut Vm) -> usize {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref(0);
    let t = pool.get_type_or_panic(type_ref);
    vm.push_single_stack_value(StackValue::new(t, Default::default()));
    1 + refs(1)
}
pub(in crate::interpreter) fn handle_ld_type(chunk: &Chunk, vm: &mut Vm) -> usize {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref(0);
    let value_ref = chunk.read_ref(1);
    let t = pool.get_type_or_panic(type_ref);
    let v = pool.get_single_or_panic(value_ref);
    vm.push_single_stack_value(StackValue::new(t, v));
    1 + refs(2)
}

pub(in crate::interpreter) fn handle_ld_true(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(true.into());
    1
}

pub(in crate::interpreter) fn handle_ld_false(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(false.into());
    1
}
