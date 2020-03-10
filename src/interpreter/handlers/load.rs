use crate::code::Chunk;
use crate::error::VmError;
use crate::interpreter::{run, InterpreterResult};
use crate::refs::refs;
use crate::stack::data::IntoStackData;
use crate::types::Type;
use crate::Vm;

pub(in crate::interpreter) fn handle_u64_ld0(_: &Chunk, vm: &mut Vm) -> InterpreterResult {
    vm.push_default_with_type(Type::U64);
    InterpreterResult::new(1)
}

pub(in crate::interpreter) fn handle_i64_ld0(_: &Chunk, vm: &mut Vm) -> InterpreterResult {
    vm.push_default_with_type(Type::I64);
    InterpreterResult::new(1)
}

pub(in crate::interpreter) fn handle_ld_unit(_: &Chunk, vm: &mut Vm) -> InterpreterResult {
    vm.push_stack_data_with_type(Default::default(), Type::Unit);
    InterpreterResult::new(1)
}

pub(in crate::interpreter) fn handle_ld_typed0(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult {
    let result = run(|| {
        let pool = vm.current_const_pool();
        let type_ref = chunk.read_ref(0).ok_or(VmError::InvalidBytecode)?;
        let t = pool.get_type(type_ref).ok_or(VmError::ConstantPoolError)?;
        vm.push_stack_data_with_type(Default::default(), t);
        Ok(())
    });
    InterpreterResult::new(1 + refs(1)).with_error_opt(result.err())
}
pub(in crate::interpreter) fn handle_ld_type(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult {
    let result = run(|| {
        let pool = vm.current_const_pool();
        let type_ref = chunk.read_ref(0).ok_or(VmError::InvalidBytecode)?;
        let value_ref = chunk.read_ref(1).ok_or(VmError::InvalidBytecode)?;
        let t = pool.get_type(type_ref).ok_or(VmError::ConstantPoolError)?;
        let v = pool
            .get_single(value_ref)
            .ok_or(VmError::ConstantPoolError)?;
        vm.push_stack_data_with_type(v, t);
        Ok(())
    });
    InterpreterResult::new(1 + refs(2)).with_error_opt(result.err())
}

pub(in crate::interpreter) fn handle_ld_true(_: &Chunk, vm: &mut Vm) -> InterpreterResult {
    vm.push_stack_data_with_type(true.into_stack_data(), Type::Bool);
    InterpreterResult::new(1)
}

pub(in crate::interpreter) fn handle_ld_false(_: &Chunk, vm: &mut Vm) -> InterpreterResult {
    vm.push_stack_data_with_type(false.into_stack_data(), Type::Bool);
    InterpreterResult::new(1)
}
