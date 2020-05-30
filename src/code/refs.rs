use serde::{Deserialize, Serialize};
use std::mem::size_of;
pub type Ref = usize;

/// Type of the reference to a stack value in bytecode
#[derive(Debug, Eq, PartialEq, Copy, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct StackRef(pub Ref);

impl From<Ref> for StackRef {
    fn from(obj: Ref) -> Self {
        Self(obj)
    }
}

/// Type of the reference to a constant pool value in bytecode
#[derive(Debug, Eq, PartialEq, Copy, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PoolRef(pub Ref);

impl From<Ref> for PoolRef {
    fn from(obj: Ref) -> Self {
        Self(obj)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CodeRef {
    Stack(StackRef),
    Pool(PoolRef),
    Offset(usize),
}

impl CodeRef {
    pub fn ref_value(self) -> Option<Ref> {
        match self {
            CodeRef::Stack(r) => Some(r.0),
            CodeRef::Pool(r) => Some(r.0),
            CodeRef::Offset(_) => None,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            CodeRef::Stack(r) => Vec::from(r.0.to_le_bytes()),
            CodeRef::Pool(r) => Vec::from(r.0.to_le_bytes()),
            CodeRef::Offset(r) => Vec::from(r.to_le_bytes()),
        }
    }
}

impl From<StackRef> for CodeRef {
    fn from(obj: StackRef) -> Self {
        CodeRef::Stack(obj)
    }
}

impl From<PoolRef> for CodeRef {
    fn from(obj: PoolRef) -> Self {
        CodeRef::Pool(obj)
    }
}

/// 3 reference opcode
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ThreeStackRefs {
    pub result: StackRef,
    pub op1: StackRef,
    pub op2: StackRef,
}
/// 2 reference opcode
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct TwoStackRefs {
    pub result: StackRef,
    pub op: StackRef,
}

/// 3 reference constructor for opcodes
pub const fn three(result: usize, op1: usize, op2: usize) -> ThreeStackRefs {
    ThreeStackRefs {
        result: StackRef(result),
        op1: StackRef(op1),
        op2: StackRef(op2),
    }
}

/// 2 reference constructor for opcodes
pub const fn two(result: usize, op: usize) -> TwoStackRefs {
    TwoStackRefs {
        result: StackRef(result),
        op: StackRef(op),
    }
}
/// 1 reference opcode constructor
pub const fn s(r: usize) -> StackRef {
    StackRef(r)
}

/// 1 reference to the constant pool constructor
pub const fn p(r: usize) -> PoolRef {
    PoolRef(r)
}
/// Return the amount of bytes `n_refs` takes in the bytecode
pub const fn refs_size(n_refs: usize) -> usize {
    n_refs * size_of::<StackRef>()
}
