use crate::code::refs::{refs_size, CodeRef};
use crate::opcodes::Opcode;
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

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
    pub tag: Cow<'static, str>,
    /// Reference to value somewhere in the vm
    pub code_ref: CodeRef,
}

impl DecoderRef {
    pub fn new_with_no_tag(code_ref: impl Into<CodeRef>) -> Self {
        Self {
            tag: "".into(),
            code_ref: code_ref.into(),
        }
    }

    pub fn new(vm_ref: impl Into<CodeRef>, tag: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tag: tag.into(),
            code_ref: vm_ref.into(),
        }
    }

    pub fn offset(r: usize, tag: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tag: tag.into(),
            code_ref: CodeRef::Offset(r),
        }
    }
}

impl Display for DecoderRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (symbol, value) = match self.code_ref {
            CodeRef::Stack(r) => ("@", r.0),
            CodeRef::Pool(r) => ("$", r.0),
            CodeRef::Offset(r) => ("*", r),
        };
        f.write_str(symbol)?;
        if self.tag.is_empty() {
            write!(f, "<{}>", self.tag)?;
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
    Four(DecoderRef, DecoderRef, DecoderRef, DecoderRef),
}

impl DecoderRefs {
    pub fn count(&self) -> usize {
        match self {
            DecoderRefs::Zero => 0,
            DecoderRefs::One(_) => 1,
            DecoderRefs::Two(_, _) => 2,
            DecoderRefs::Three(_, _, _) => 3,
            DecoderRefs::Four(_, _, _, _) => 4,
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        // TODO: possible optimization with the allocated size
        let mut res = Vec::new();
        match self {
            DecoderRefs::Zero => {}
            DecoderRefs::One(r) => {
                res.extend(r.code_ref.to_bytes());
            }
            DecoderRefs::Two(r1, r2) => {
                res.extend(r1.code_ref.to_bytes());
                res.extend(r2.code_ref.to_bytes());
            }
            DecoderRefs::Three(r1, r2, r3) => {
                res.extend(r1.code_ref.to_bytes());
                res.extend(r2.code_ref.to_bytes());
                res.extend(r3.code_ref.to_bytes());
            }
            DecoderRefs::Four(r1, r2, r3, r4) => {
                res.extend(r1.code_ref.to_bytes());
                res.extend(r2.code_ref.to_bytes());
                res.extend(r3.code_ref.to_bytes());
                res.extend(r4.code_ref.to_bytes());
            }
        }
        res
    }
}

impl Display for DecoderRefs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DecoderRefs::Zero => Ok(()),
            DecoderRefs::One(r) => write!(f, "{}", r),
            DecoderRefs::Two(r1, r2) => write!(f, "{} {}", r1, r2),
            DecoderRefs::Three(r1, r2, r3) => write!(f, "{} {} {}", r1, r2, r3),
            DecoderRefs::Four(r1, r2, r3, r4) => write!(f, "{} {} {} {}", r1, r2, r3, r4),
        }
    }
}

impl DecodedOpcode {
    pub(crate) fn new(op_code: Opcode, refs: DecoderRefs) -> Self {
        Self {
            consumed: op_code.size() + refs_size(refs.count()),
            op_code,
            refs,
        }
    }

    pub(crate) fn zero(op_code: Opcode) -> Self {
        Self {
            consumed: op_code.size(),
            op_code,
            refs: DecoderRefs::Zero,
        }
    }

    pub(crate) fn one(op_code: Opcode, rf: DecoderRef) -> Self {
        Self::new(op_code, DecoderRefs::One(rf))
    }
}
