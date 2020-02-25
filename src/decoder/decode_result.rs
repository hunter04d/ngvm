use crate::opcodes::{refs, Opcode};
use std::fmt::{self, Display, Formatter};

/// The result of the decoding the input stream
pub(crate) struct DecodeResult {
    /// number of bytes consumed
    pub(crate) consumed: usize,
    /// String repr of the opcode
    pub(crate) repr: String,
}

pub(crate) struct PoolRef(usize);

impl From<usize> for PoolRef {
    fn from(obj: usize) -> Self {
        PoolRef(obj)
    }
}

impl Display for PoolRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

pub(crate) struct StackRef(usize);

impl From<usize> for StackRef {
    fn from(obj: usize) -> Self {
        StackRef(obj)
    }
}

impl Display for StackRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

impl DecodeResult {
    #[allow(dead_code)]
    pub(crate) fn new(consumed: usize, repr: String) -> Self {
        Self { consumed, repr }
    }

    pub(crate) fn with_refs(ref_count: usize, repr: String) -> Self {
        Self {
            consumed: 1 + refs(ref_count),
            repr,
        }
    }

    pub(crate) fn from_no_refs(op: Opcode) -> Self {
        Self {
            consumed: 1,
            repr: format!("{:?}", op),
        }
    }
}
