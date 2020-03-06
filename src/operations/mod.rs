
mod blankets;
pub mod markers;
pub trait BiOpMarker {}

pub trait BiOp<'a, M : BiOpMarker, Other = &'a Self> {
    type Output;

    fn invoke(&self, other: Other) -> Self::Output;
}

pub trait UOpMarker {}

pub trait UOp<M : UOpMarker> {
    type Output;

    fn invoke(&self) -> Self::Output;
}
