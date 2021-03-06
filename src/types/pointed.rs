use std::fmt::{self, Display, Formatter};

use crate::code::refs::StackRef;
use crate::stack::data::{IntoPrimitive, StackData};
use crate::vm::refs::LocatedRef;
use crate::vm::ValueLocation;

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

    pub fn reference(pointer: impl Into<VmType>, kind: RefKind, location: RefLocation) -> Self {
        PointedType::Ref(RefType {
            kind,
            points_to: location,
            pointer: pointer.into(),
        })
    }

    pub fn ref_reference(pointer: impl Into<VmType>, location: RefLocation) -> Self {
        Self::reference(pointer, RefKind::Ref, location)
    }

    pub fn mut_reference(pointer: impl Into<VmType>, location: RefLocation) -> Self {
        Self::reference(pointer, RefKind::Mut, location)
    }

    pub fn size(&self) -> usize {
        match self {
            PointedType::SArr(SArrType { len, pointer }) => len * pointer.size(),
            PointedType::Ref(_) => 1,
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

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum RefLocation {
    Stack,
    Heap,
    TransientOnStack,
    TransientOnHeap,
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct RefType {
    pub kind: RefKind,
    pub points_to: RefLocation,
    pub pointer: VmType,
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct SArrType {
    pub len: usize,
    pub pointer: VmType,
}

impl RefType {
    pub fn locate(&self, ref_value: &StackData) -> LocatedRef {
        let index: usize = ref_value.into_primitive();
        match self.points_to {
            RefLocation::Stack => LocatedRef::Stack(StackRef(index)),
            RefLocation::Heap => unimplemented!(),
            RefLocation::TransientOnStack => LocatedRef::Transient(ValueLocation::Stack(index)),
            RefLocation::TransientOnHeap => unreachable!(),
        }
    }

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
