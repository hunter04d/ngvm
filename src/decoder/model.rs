use crate::refs::{refs_size, VmRef, PoolRef, StackRef};
use std::fmt::{self, Display, Formatter};
use crate::opcodes::Opcode;
use std::borrow::Cow;

/// The result of the decoding the input stream
pub struct DecodedOpcode {
    /// number of bytes consumed
    pub(crate) consumed: usize,
    /// opcode
    pub(crate) op_code: Opcode,
    /// refs that the opcode contains
    pub(crate) refs: DecoderRefs,
}

pub struct DecoderRef {
    /// Tag of reference, each opcode is free to interpret it as it pleases
    ///
    /// Tag generally signifies what the reference means
    /// Common tags can be found in [super::tags](tags module)
    pub tag: Option<Cow<'static, str>>,
    /// Reference to value somewhere in the vm
    pub vm_ref: VmRef,
}

impl DecoderRef {
    pub fn pool(pool_ref: PoolRef) -> Self {
        Self { tag: None, vm_ref: pool_ref.into() }
    }

    pub fn stack(stack_ref: StackRef) -> Self {
        Self { tag: None, vm_ref: stack_ref.into() }
    }

    pub fn pool_with_tag(pool_ref: PoolRef, tag: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tag: Some(tag.into()),
            vm_ref: pool_ref.into(),
        }
    }

    pub fn stack_with_tag(stack_ref: StackRef, tag: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tag: Some(tag.into()),
            vm_ref: stack_ref.into(),
        }
    }
}

impl Display for DecoderRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (symbol, value) = match self.vm_ref {
            VmRef::Stack(r) => ("@", r.0),
            VmRef::Pool(r) => ("$", r.0),
        };
        f.write_str(symbol)?;
        if let Some(tag) = &self.tag {
            write!(f, "<{}>", *tag)?;
        }
        write!(f, "{}", value)?;
        Ok(())
    }
}

pub enum DecoderRefs {
    Zero,
    One(DecoderRef),
    Two(DecoderRef, DecoderRef),
    Three(DecoderRef, DecoderRef, DecoderRef),
}

impl DecoderRefs {
    pub fn count(&self) -> usize {
        match self {
            DecoderRefs::Zero => 0,
            DecoderRefs::One(_) => 1,
            DecoderRefs::Two(_, _) => 2,
            DecoderRefs::Three(_, _, _) => 3,
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        match self {
            DecoderRefs::Zero => {},
            DecoderRefs::One(r) => {
                res.extend_from_slice(&r.vm_ref.value().to_le_bytes());
            },
            DecoderRefs::Two(r1, r2 ) => {
                res.extend_from_slice(&r1.vm_ref.value().to_le_bytes());
                res.extend_from_slice(&r2.vm_ref.value().to_le_bytes());
            },
            DecoderRefs::Three(r1, r2, r3) => {
                res.extend_from_slice(&r1.vm_ref.value().to_le_bytes());
                res.extend_from_slice(&r2.vm_ref.value().to_le_bytes());
                res.extend_from_slice(&r3.vm_ref.value().to_le_bytes());
            },
        }
        res
    }
}

impl Display for DecoderRefs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DecoderRefs::Zero =>
                Ok(()),
            DecoderRefs::One(r) =>
                write!(f, "{}", r),
            DecoderRefs::Two(r1, r2) =>
                write!(f, "{} {}", r1, r2),
            DecoderRefs::Three(r1, r2, r3) =>
                write!(f, "{} {} {}", r1, r2, r3),
        }
    }
}

impl DecodedOpcode {
    #[allow(dead_code)]
    pub(crate) fn new(consumed: usize, op_code: Opcode, refs: DecoderRefs) -> Self {
        Self { consumed, op_code, refs }
    }

    #[allow(dead_code)]
    pub(crate) fn single(op_code: Opcode, refs: DecoderRefs) -> Self {
        Self { consumed: 1 + refs_size(refs.count()), op_code, refs }
    }

    #[allow(dead_code)]
    pub(crate) fn double(op_code: Opcode, refs: DecoderRefs) -> Self {
        Self { consumed: 2 + refs_size(refs.count()), op_code, refs }
    }
}
