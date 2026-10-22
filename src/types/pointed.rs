use std::fmt::{self, Display, Formatter};

use num_derive::FromPrimitive;

use super::VmType;

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum PointedType {
    SArr(SArrType),
    Ref(RefType),
    Boxed(VmType),
}

impl PointedType {
    pub fn s_arr(pointer: impl Into<VmType>, len: usize) -> Self {
        PointedType::SArr(SArrType {
            len,
            pointer: pointer.into(),
        })
    }

    pub fn reference(pointer: impl Into<VmType>, kind: RefKind) -> Self {
        PointedType::Ref(RefType {
            kind,
            pointer: pointer.into(),
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
            PointedType::SArr(SArrType { len, pointer }) => len * pointer.size(),
            PointedType::Ref(_) => 2,
            PointedType::Boxed(_) => 1,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum RefKind {
    /// Immutable reference
    Ref,
    /// Mutable reference
    Mut,
}

/// Where a reference points to
///
/// This is not a part of the reference type, but of the reference value: references of the same
/// type can point to different locations. See [`RefData`](crate::vm::refs::RefData).
#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, FromPrimitive)]
pub enum RefLocation {
    Stack,
    Heap,
    TransientOnStack,
    TransientOnHeap,
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct RefType {
    pub kind: RefKind,
    pub pointer: VmType,
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct SArrType {
    pub len: usize,
    pub pointer: VmType,
}

impl RefType {
    pub fn is_copy(&self) -> bool {
        match self.kind {
            RefKind::Mut => false,
            RefKind::Ref => true,
        }
    }
}

impl Display for RefType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.kind {
                RefKind::Mut => "&mut",
                RefKind::Ref => "&",
            }
        )?;
        match &self.pointer {
            VmType::Primitive(p) => write!(f, "{:?}", p),
            VmType::PointedType(p) => match p.as_ref() {
                PointedType::SArr(SArrType { len, pointer }) => {
                    write!(f, "[{:?};{}]", pointer, len)
                }
                PointedType::Ref(r) => write!(f, "({})", r),
                PointedType::Boxed(t) => write!(f, "Box<{:?}>", t)
            },
        }
    }
}
