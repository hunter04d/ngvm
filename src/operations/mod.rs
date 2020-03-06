use std::fmt::Debug;

pub mod blankets;
pub mod markers;
pub trait BiOpMarker: Debug {}

pub trait BiOp<M: BiOpMarker, Other = Self> {
    type Output;

    fn invoke(self, other: Other) -> Self::Output;
}

pub trait UOpMarker {}

pub trait UOp<M: UOpMarker> {
    type Output;

    fn invoke(self) -> Self::Output;
}
