use crate::code::Chunk;
use crate::refs::refs;
use crate::stack::data::IntoStackData;
use crate::types::Type;
use crate::Vm;

pub(in crate::interpreter) fn handle_u64_ld0(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_stack_data_with_type(Default::default(), Type::U64);
    1
}

pub(in crate::interpreter) fn handle_i64_ld0(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_stack_data_with_type(Default::default(), Type::I64);
    1
}

pub(in crate::interpreter) fn handle_ld_unit(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_stack_data_with_type(Default::default(), Type::Unit);
    1
}

pub(in crate::interpreter) fn handle_ld_typed0(chunk: &Chunk, vm: &mut Vm) -> usize {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref(0).expect("Invalid Bytecode");
    let t = pool.get_type_or_panic(type_ref);
    vm.push_stack_data_with_type(Default::default(), t);
    1 + refs(1)
}
pub(in crate::interpreter) fn handle_ld_type(chunk: &Chunk, vm: &mut Vm) -> usize {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref(0).unwrap();
    let value_ref = chunk.read_ref(1).unwrap();
    let t = pool.get_type_or_panic(type_ref);
    let v = pool.get_single_or_panic(value_ref);
    vm.push_stack_data_with_type(v, t);
    1 + refs(2)
}

pub(in crate::interpreter) fn handle_ld_true(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_stack_data_with_type(true.into_stack_data(), Type::Bool);
    1
}

pub(in crate::interpreter) fn handle_ld_false(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_stack_data_with_type(false.into_stack_data(), Type::Bool);
    1
}
