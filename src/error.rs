use crate::types::checker::{TaggedType, TypeError};
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
}
