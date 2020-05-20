use std::mem::size_of;

pub type Ref = usize;

/// Type of the reference to a stack value in bytecode
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct StackRef(pub Ref);

impl From<Ref> for StackRef {
    fn from(obj: Ref) -> Self {
        Self(obj)
    }
}

/// Type of the reference to a constant pool value in bytecode
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PoolRef(pub Ref);

impl From<Ref> for PoolRef {
    fn from(obj: Ref) -> Self {
        Self(obj)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum VmRef {
    Stack(StackRef),
    Pool(PoolRef),
    Offset(usize),
}

impl VmRef {
    pub fn ref_value(self) -> Option<Ref> {
        match self {
            VmRef::Stack(r) => Some(r.0),
            VmRef::Pool(r) => Some(r.0),
            VmRef::Offset(_) => None,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            VmRef::Stack(r) => Vec::from(r.0.to_le_bytes()),
            VmRef::Pool(r) => Vec::from(r.0.to_le_bytes()),
            VmRef::Offset(r) => Vec::from(r.to_le_bytes()),
        }
    }
}

impl From<StackRef> for VmRef {
    fn from(obj: StackRef) -> Self {
        VmRef::Stack(obj)
    }
}

impl From<PoolRef> for VmRef {
    fn from(obj: PoolRef) -> Self {
        VmRef::Pool(obj)
    }
}

/// 3 reference opcode
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ThreeStackRefs {
    pub result: StackRef,
    pub op1: StackRef,
    pub op2: StackRef,
}
/// 2 reference opcode
#[derive(Debug, Eq, PartialEq, Clone)]
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
pub const fn one(r: usize) -> StackRef {
    StackRef(r)
}

/// Return the amount of bytes `n_refs` takes in the bytecode
pub const fn refs_size(n_refs: usize) -> usize {
    n_refs * size_of::<StackRef>()
}
