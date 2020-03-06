use crate::opcodes::Ref;
use std::mem::size_of;

/// 3 reference opcode
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ThreeRefs {
    pub result: Ref,
    pub op1: Ref,
    pub op2: Ref,
}
/// 2 reference opcode
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TwoRefs {
    pub result: Ref,
    pub op: Ref,
}

/// 3 reference constructor for opcodes
pub const fn three(result: Ref, op1: Ref, op2: Ref) -> ThreeRefs {
    ThreeRefs { result, op1, op2 }
}

/// 3 reference constructor for opcodes
pub const fn two(result: Ref, op: Ref) -> TwoRefs {
    TwoRefs { result, op }
}

/// Return the amount of bytes `n_refs` takes in the bytecode
pub const fn refs(n_refs: Ref) -> usize {
    n_refs * size_of::<Ref>()
}
