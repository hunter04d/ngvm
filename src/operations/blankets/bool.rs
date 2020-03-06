use super::super::markers::*;
use super::super::BiOp;

impl<T: PartialEq> BiOp<Eq> for T {
    type Output = bool;

    fn invoke(self, other: Self) -> Self::Output {
        self.eq(&other)
    }
}

impl<T: PartialEq> BiOp<Ne> for T {
    type Output = bool;

    fn invoke(self, other: Self) -> Self::Output {
        self.ne(&other)
    }
}

impl<T: PartialOrd> BiOp<Lt> for T {
    type Output = bool;

    fn invoke(self, other: Self) -> Self::Output {
        self.lt(&other)
    }
}

impl<T: PartialOrd> BiOp<Le> for T {
    type Output = bool;

    fn invoke(self, other: Self) -> Self::Output {
        self.le(&other)
    }
}

impl<T: PartialOrd> BiOp<Gt> for T {
    type Output = bool;

    fn invoke(self, other: Self) -> Self::Output {
        self.gt(&other)
    }
}

impl<T: PartialOrd> BiOp<Ge> for T {
    type Output = bool;

    fn invoke(self, other: Self) -> Self::Output {
        self.ge(&other)
    }
}
