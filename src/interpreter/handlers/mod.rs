use crate::code::Chunk;
use crate::error::VmError;
use crate::vm::{Vm, VmRefSource};

use super::stack_tracer::StackTracer;
use crate::code::refs::refs_size;

pub(in crate::interpreter) mod alu;
pub(in crate::interpreter) mod array;
pub(in crate::interpreter) mod jumps;
pub(in crate::interpreter) mod load;
pub(in crate::interpreter) mod memory;
pub(in crate::interpreter) mod stack;

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

    unimplemented!(
        "Wide opcodes are not supported yet (@{}) {}",
        chunk.offset(),
        new_chunk.read_byte(0).ok_or(VmError::InvalidBytecode)?
    )
}

pub(crate) fn noop(chunk: &Chunk, _vm: &mut Vm) -> Result<usize, VmError> {
    panic!(
        "a bad opcode detected (@{}){}",
        chunk.offset(),
        chunk.read_byte(0).ok_or(VmError::InvalidBytecode)?
    );
}
