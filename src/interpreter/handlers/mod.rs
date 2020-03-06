use super::stack_tracer::StackTracer;
use crate::code::Chunk;
use crate::refs::refs;
use crate::Vm;

pub(in crate::interpreter) mod alu;
pub(in crate::interpreter) mod load;

/// For debug only
pub(super) fn handle_trace_stack_value(chunk: &Chunk, vm: &mut Vm) -> usize {
    let stack_ref = chunk.read_ref(0).unwrap();
    let meta = vm.stack_metadata(stack_ref).unwrap();
    eprintln!(
        "Trace @{}: {:#?}",
        stack_ref,
        StackTracer(&vm.stack[meta.index..], meta)
    );
    1 + refs(1)
}

pub(super) fn handle_wide(chunk: &Chunk, vm: &mut Vm) -> usize {
    noop(chunk, vm)
}

pub(crate) fn noop(chunk: &Chunk, _vm: &mut Vm) -> usize {
    panic!("a bad opcode detected <{}>", chunk.opcode_value());
}
