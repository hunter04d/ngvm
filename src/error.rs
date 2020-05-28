use crate::opcodes::Opcode;
use crate::refs::StackRef;
use crate::types::checker::{TaggedType, TypeError};
use crate::types::RefKind;
use crate::vm::lock::LockError;
use thiserror::Error;

/// Represents an error that originated inside the vm internal logic
#[derive(Error, Debug)]
pub enum VmError {
    #[error("bad bytecode")]
    InvalidBytecode,
    #[error("bad vm state, cannot continue")]
    BadVmState,
    #[error("The operation is not supported for type {0:?}")]
    InvalidTypeForOperation(TaggedType),
    #[error("Error while processing a binary operation")]
    BiOpError,
    #[error("Error while processing a unary operation")]
    UOpError,
    #[error("Constant pool constrains invalid value")]
    ConstantPoolError,
    #[error("Type error: {0:?}")]
    TypeError(Vec<TypeError>),
    #[error(
        "Attempt to take the reference of kind {0:?} in the same vm cycle as the object @{1:?}"
    )]
    SameCycleRef(RefKind, StackRef),

    #[error("{0} (@{1:?})")]
    LockError(LockError, StackRef),
}

#[derive(Debug)]
pub struct VmContextError {
    pub error: VmError,
    pub location: Option<usize>,
    pub opcode: Option<Opcode>,
}

impl From<VmError> for VmContextError {
    fn from(obj: VmError) -> Self {
        VmContextError {
            error: obj,
            location: None,
            opcode: None,
        }
    }
}
