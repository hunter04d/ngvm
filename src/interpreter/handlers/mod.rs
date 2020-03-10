use super::stack_tracer::StackTracer;
use crate::code::Chunk;
use crate::error::VmError;
use crate::interpreter::{run, InterpreterResult};
use crate::refs::refs;
use crate::Vm;

pub(in crate::interpreter) mod alu;
pub(in crate::interpreter) mod load;

/// For debug only
pub(super) fn handle_trace_stack_value(chunk: &Chunk, vm: &mut Vm) -> InterpreterResult {
    let error = run(|| {
        let stack_ref = chunk.read_ref(0).ok_or(VmError::InvalidBytecode)?;
        let meta = vm.stack_metadata(stack_ref)?;
        eprintln!(
            "Trace @{}: {:#?}",
            stack_ref,
            // TODO: one pointer types are a thing rethink the StackTracer interface
            StackTracer(&vm.stack[meta.index..], meta)
        );
        Ok(())
    });

    InterpreterResult::new(1 + refs(1)).with_error_opt(error.err())
}

pub(super) fn handle_wide(chunk: &Chunk, _: &mut Vm) -> InterpreterResult {
    let mut new_chunk = chunk.clone();
    new_chunk.advance(1);

    unimplemented!("Wide opcodes are not supporsed yet (@{})", chunk.offset())
}

pub(crate) fn noop(chunk: &Chunk, _vm: &mut Vm) -> InterpreterResult {
    panic!("a bad opcode detected <{}>", chunk.opcode_value());
}
