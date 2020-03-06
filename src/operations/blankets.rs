use num_traits::ops::checked as cops;
use crate::operations::*;

impl <T: cops::CheckedAdd> BiOp<'_, CheckedAdd> for T {
    type Output = Option<T::Output>;

    fn invoke(&self, other: &Self) -> Self::Output {
        self.checked_add(other)
    }
}

impl <T: cops::CheckedSub> BiOp<'_, CheckedSub> for T {
    type Output = Option<T::Output>;

    fn invoke(&self, other: &Self) -> Self::Output {
        self.checked_sub(other)
    }
}

impl <T: cops::CheckedMul> BiOp<'_, CheckedMul> for T {
    type Output = Option<T::Output>;

    fn invoke(&self, other: &Self) -> Self::Output {
        self.checked_mul(other)
    }
}

impl <T: cops::CheckedDiv> BiOp<'_, CheckedDiv> for T {
    type Output = Option<T::Output>;

    fn invoke(&self, other: &Self) -> Self::Output {
        self.checked_div(other)
    }
}

impl <T: cops::CheckedRem> BiOp<'_, CheckedRem> for T {
    type Output = Option<T::Output>;

    fn invoke(&self, other: &Self) -> Self::Output {
        self.checked_rem(other)
    }
}

impl <T: cops::CheckedShl> BiOp<'_, CheckedShl, u32> for T {
    type Output = Option<T::Output>;

    fn invoke(&self, other: u32) -> Self::Output {
        self.checked_shl(other)
    }
}

impl <T: cops::CheckedShr> BiOp<'_, CheckedShr, u32> for T {
    type Output = Option<T::Output>;

    fn invoke(&self, other: u32) -> Self::Output {
        self.checked_shr(other)
    }
}

impl <T: cops::CheckedNeg> UOp<CheckedNeg> for T {
    type Output = Option<Self>;

    fn invoke(&self) -> Self::Output {
        self.checked_neg()
    }
}
