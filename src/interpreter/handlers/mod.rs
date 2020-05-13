use super::stack_tracer::StackTracer;
use crate::code::Chunk;
use crate::refs::refs_size;
use crate::vm::{Vm, VmRefSource};
use crate::error::VmError;
pub(in crate::interpreter) mod alu;
pub(in crate::interpreter) mod load;

/// For debug only
pub(super) fn handle_trace_stack_value(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let stack_ref = chunk.read_ref_stack_vm(0)?;
    let meta = vm.stack_metadata(stack_ref)?;
    eprintln!(
        "Trace {:?}: {:#?}",
        stack_ref,
        // TODO: one pointer types are a thing rethink the StackTracer interface
        StackTracer(&vm.stack[meta.index.0..], meta)
    );
    Ok(1 + refs_size(1))
}

pub(super) fn handle_wide(chunk: &Chunk, _: &mut Vm) -> Result<usize, VmError> {
    let mut new_chunk = chunk.clone();
    new_chunk.advance(1);

    unimplemented!("Wide opcodes are not supporsed yet (@{})", chunk.offset())
}

pub(crate) fn noop(chunk: &Chunk, _vm: &mut Vm) -> Result<usize, VmError> {
    panic!("a bad opcode detected <{}>", chunk.read_byte());
}
