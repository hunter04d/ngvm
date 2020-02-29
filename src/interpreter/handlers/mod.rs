use super::stack_tracer::StackTracer;
use crate::code::Chunk;
use crate::opcodes::refs;
use crate::Vm;

pub(in crate::interpreter) mod load;
pub(in crate::interpreter) mod math;

/// For debug only
pub(super) fn handle_trace_stack_value(chunk: &Chunk, vm: &mut Vm) -> usize {
    let stack_ref = chunk.read_ref(0);
    eprintln!(
        "Trace @{}: {:#?}",
        stack_ref,
        StackTracer(&vm.stack[stack_ref..])
    );
    1 + refs(1)
}

pub(super) fn handle_wide(chunk: &Chunk, vm: &mut Vm) -> usize {
    noop(chunk, vm)
}

pub(crate) fn noop(chunk: &Chunk, _vm: &mut Vm) -> usize {
    panic!("a bad opcode detected <{}>", chunk.opcode_value());
}
