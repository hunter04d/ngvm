#![doc(hidden)]

use std::ops;

use super::markers::*;
use super::{BiOp, UOp};

pub mod checked;
pub mod logical;
pub mod shifts;

impl<T: ops::Add> BiOp<Add> for T {
    type Output = T::Output;

    fn invoke(self, other: Self) -> Self::Output {
        self.add(other)
    }
}

impl<T: ops::Sub> BiOp<Sub> for T {
    type Output = T::Output;

    fn invoke(self, other: Self) -> Self::Output {
        self.sub(other)
    }
}

impl<T: ops::Mul> BiOp<Mul> for T {
    type Output = T::Output;

    fn invoke(self, other: Self) -> Self::Output {
        self.mul(other)
    }
}

impl<T: ops::Div> BiOp<Div> for T {
    type Output = T::Output;

    fn invoke(self, other: Self) -> Self::Output {
        self.div(other)
    }
}

impl<T: ops::Rem> BiOp<Rem> for T {
    type Output = T::Output;

    fn invoke(self, other: Self) -> Self::Output {
        self.rem(other)
    }
}

impl<T: ops::Neg> UOp<Neg> for T {
    type Output = T::Output;

    fn invoke(self) -> Self::Output {
        self.neg()
    }
}
