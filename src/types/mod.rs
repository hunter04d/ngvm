use std::convert::Into;

pub mod checker;
mod primitive;
pub use primitive::{HasPrimitiveType, PrimitiveType};
use smallvec::smallvec;
use smallvec::SmallVec;

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
}

impl From<PrimitiveType> for VmType {
    fn from(obj: PrimitiveType) -> Self {
        VmType::Primitive(obj)
    }
}

#[repr(u8)]
#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum PointedKind {
    Ref,
    Arr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PointedType {
    pub kind: PointedKind,
    pub pointer: SmallVec<[VmType; 1]>,
}

impl PointedType {
    pub fn primitive_arr(pointer: impl Into<VmType>) -> Self {
        Self {
            pointer: smallvec![pointer.into()],
            kind: PointedKind::Arr,
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
