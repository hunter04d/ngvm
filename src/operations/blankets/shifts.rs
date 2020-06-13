use std::ops;

use num_traits::ops::checked as cops;
use num_traits::PrimInt;

use crate::operations::markers::*;
use crate::operations::BiOp;

impl<T: ops::Shl<u32>> BiOp<Shl, u32> for T {
    type Output = T::Output;

    fn invoke(self, other: u32) -> Self::Output {
        self.shl(other)
    }
}

impl<T: ops::Shr<u32>> BiOp<Shr, u32> for T {
    type Output = T::Output;

    fn invoke(self, other: u32) -> Self::Output {
        self.shr(other)
    }
}

impl<T: PrimInt> BiOp<RotL, u32> for T {
    type Output = T;

    fn invoke(self, other: u32) -> Self::Output {
        self.rotate_left(other)
    }
}

impl<T: PrimInt> BiOp<RotR, u32> for T {
    type Output = T;

    fn invoke(self, other: u32) -> Self::Output {
        self.rotate_right(other)
    }
}

impl<T: cops::CheckedShl> BiOp<CheckedShl, u32> for T {
    type Output = Option<T::Output>;

    fn invoke(self, other: u32) -> Self::Output {
        self.checked_shl(other)
    }
}

impl<T: cops::CheckedShr> BiOp<CheckedShr, u32> for T {
    type Output = Option<T::Output>;

    fn invoke(self, other: u32) -> Self::Output {
        self.checked_shr(other)
    }
}

impl<T: PrimInt> BiOp<CheckedRotL, u32> for T {
    type Output = Option<T>;

    fn invoke(self, other: u32) -> Self::Output {
        Some(self.rotate_left(other))
    }
}

impl<T: PrimInt> BiOp<CheckedRotR, u32> for T {
    type Output = Option<T>;

    fn invoke(self, other: u32) -> Self::Output {
        Some(self.rotate_right(other))
    }
}
