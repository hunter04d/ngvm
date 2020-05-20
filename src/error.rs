use crate::opcodes::Opcode;
use crate::types::Type;
use thiserror::Error;

/// Represents an error that originated inside the vm internal logic
#[derive(Error, Debug)]
pub enum VmError {
    #[error("bad bytecode")]
    InvalidBytecode,
    #[error("bad vm state, cannot continue")]
    BadVmState,
    #[error("The operation {0:?} is not supported for type {1:?}")]
    InvalidTypeForOperation(Opcode, Type),
    #[error("Operands have mismatched types. Opcode {0:?} does not work with {1:?} and {2:?}")]
    OperandsTypeMismatch(Opcode, Type, Type),
    #[error("Output types mismatch")]
    OutputTypeMismatch(Opcode, Type),
    #[error("Error while processing a binary operation")]
    BiOpError,
    #[error("Error while processing a unary operation")]
    UOpError,
    #[error("Constant pool constrains invalid value")]
    ConstantPoolError,
}
