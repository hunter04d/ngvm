use super::super::markers::*;
use super::super::{BiOp, UOp};
use num_traits::ops::checked as cops;

impl<T: cops::CheckedAdd> BiOp<CheckedAdd> for T {
    type Output = Option<T::Output>;

    fn invoke(self, other: Self) -> Self::Output {
        self.checked_add(&other)
    }
}

impl<T: cops::CheckedSub> BiOp<CheckedSub> for T {
    type Output = Option<T::Output>;

    fn invoke(self, other: Self) -> Self::Output {
        self.checked_sub(&other)
    }
}

impl<T: cops::CheckedMul> BiOp<CheckedMul> for T {
    type Output = Option<T::Output>;

    fn invoke(self, other: Self) -> Self::Output {
        self.checked_mul(&other)
    }
}

impl<T: cops::CheckedDiv> BiOp<CheckedDiv> for T {
    type Output = Option<T::Output>;

    fn invoke(self, other: Self) -> Self::Output {
        self.checked_div(&other)
    }
}

impl<T: cops::CheckedRem> BiOp<CheckedRem> for T {
    type Output = Option<T::Output>;

    fn invoke(self, other: Self) -> Self::Output {
        self.checked_rem(&other)
    }
}

impl<T: cops::CheckedNeg> UOp<CheckedNeg> for T {
    type Output = Option<Self>;

    fn invoke(self) -> Self::Output {
        self.checked_neg()
    }
}
