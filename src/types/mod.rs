use std::convert::Into;

pub mod checker;
mod primitive;
pub use primitive::{HasPrimitiveType, PrimitiveType};
use std::fmt::Display;
use smallvec::alloc::fmt::Formatter;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum VmType {
    Primitive(PrimitiveType),
    PointedType(Box<PointedType>),
}

impl VmType {
    pub fn is_primitive(&self) -> bool {
        matches!(self, VmType::Primitive(_))
    }

    pub fn primitive(&self) -> Option<PrimitiveType> {
        if let VmType::Primitive(t) = self {
            Some(*t)
        } else {
            None
        }
    }

    pub fn pointed(self) -> Option<PointedType> {
        if let VmType::PointedType(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn size(&self) -> usize {
        match self {
            VmType::Primitive(p) => p.size(),
            VmType::PointedType(p) => p.size(),
        }
    }
}

impl From<PrimitiveType> for VmType {
    fn from(obj: PrimitiveType) -> Self {
        VmType::Primitive(obj)
    }
}

impl From<PointedType> for VmType {
    fn from(obj: PointedType) -> Self {
        VmType::PointedType(Box::new(obj))
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum RefKind {
    /// Mutable reference
    Mut,
    /// Immutable reference
    Ref,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RefType {
    pub kind: RefKind,
    pub pointer: VmType,
}

impl Display for RefType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self.kind {
            RefKind::Mut => "&mut",
            RefKind::Ref => "&",
        })?;
        match &self.pointer {
            VmType::Primitive(p) => {
                write!(f, "{:?}", p)
            },
            VmType::PointedType(p) => {
                match p.as_ref() {
                    PointedType::Arr { len, pointer } => write!(f, "[{:?};{}]", pointer, len),
                    PointedType::Ref(r) =>{
                        write!(f, "({})", r)
                    } ,
                }
            },
        }
    }

}

#[derive(Debug, PartialEq, Clone)]
pub enum PointedType {
    Arr { len: usize, pointer: VmType },
    Ref(RefType),
}

impl PointedType {
    pub fn arr(pointer: impl Into<VmType>, len: usize) -> Self {
        PointedType::Arr {
            len,
            pointer: pointer.into(),
        }
    }

    pub fn reference(pointer: impl Into<VmType>, kind: RefKind) -> Self {
        PointedType::Ref(RefType {
            kind,
            pointer: pointer.into()
        })
    }

    pub fn ref_reference(pointer: impl Into<VmType>) -> Self {
        Self::reference(pointer, RefKind::Ref)
    }

    pub fn mut_reference(pointer: impl Into<VmType>) -> Self {
        Self::reference(pointer, RefKind::Mut)
    }

    pub fn size(&self) -> usize {
        match self {
            PointedType::Arr { len, pointer } => len * pointer.size(),
            PointedType::Ref { .. } => 1,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ThreePrimitiveTypes {
    pub result: PrimitiveType,
    pub op1: PrimitiveType,
    pub op2: PrimitiveType,
}

#[derive(Debug, PartialEq)]
pub struct TwoPrimitiveTypes {
    pub result: PrimitiveType,
    pub op: PrimitiveType,
}
