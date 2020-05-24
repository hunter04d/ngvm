use crate::operations::markers::*;
use crate::operations::{BiOp, UOp};

impl<T: std::ops::Not> UOp<Not> for T {
    type Output = T::Output;

    fn invoke(self) -> Self::Output {
        self.not()
    }
}

impl<T: std::ops::BitAnd> BiOp<And> for T {
    type Output = T::Output;

    fn invoke(self, other: T) -> Self::Output {
        self.bitand(other)
    }
}

impl<T: std::ops::BitOr> BiOp<Or> for T {
    type Output = T::Output;

    fn invoke(self, other: T) -> Self::Output {
        self.bitor(other)
    }
}

impl<T: std::ops::BitXor> BiOp<Xor> for T {
    type Output = T::Output;

    fn invoke(self, other: T) -> Self::Output {
        self.bitxor(other)
    }
}
