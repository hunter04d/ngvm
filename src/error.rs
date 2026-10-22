use thiserror::Error;

use crate::code::refs::StackRef;
use crate::opcodes::Opcode;
use crate::types::checker::{TaggedType, TypeError};
use crate::types::RefKind;
use crate::vm::lock::LockError;
use crate::vm::ValueLocation;

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
        "Attempt to take a reference of kind {:?} in the same vm cycle as the object @{}", .0, (.1).0
    )]
    SameCycleRef(RefKind, StackRef),
    #[error(
        "Attempt to take a {:?} reference to a temporary @{}", .0, (.1).0
    )]
    RefToTemp(RefKind, StackRef),

    #[error("{0} (@{1:?})")]
    LockError(LockError, ValueLocation),
    #[error("Use of moved value @{}", (.0).0)]
    UseOfMovedValue(StackRef),
    #[error("Index {index} is out of bounds of array of length {len}")]
    IndexOutOfBounds { index: usize, len: usize },
    #[error("Attempt to store a reference in @{} which outlives the value it points to", (.0).0)]
    ReferenceEscapesScope(StackRef),
    #[error("Attempt to end a scope while a deref started in it is active")]
    ScopeEndedDuringDeref,
    #[error("Attempt to deref @{} which points to a value holding references", (.0).0)]
    DerefOfBorrowingValue(StackRef),
    #[error("Unknown function {name:?} in module {module:?}")]
    UnknownFunction { module: String, name: String },
    #[error("Function takes {expected} arguments, but the stack frame only has {got} values")]
    NotEnoughArguments { expected: usize, got: usize },
    #[error("Attempt to pass a value that is being dereferenced to a function")]
    DerefPassedToCall,
    #[error("Attempt to return outside of a function")]
    RetOutsideFunction,
    #[error("Function returns {expected} values, but {got} were returned")]
    WrongReturnCount { expected: usize, got: usize },
    #[error("Function ended without returning")]
    MissingReturn,
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
