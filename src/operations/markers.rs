//! Markers for the operations.

use super::{BiOpMarker, UOpMarker};
macro_rules! gen_markers {
    ($($t: ident),*) => {
        $(
        #[derive(Debug)]
        pub struct $t;

        impl BiOpMarker for $t {}
        )*
    };
}

gen_markers! {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    RotL,
    RotR,
    // checked
    CheckedAdd,
    CheckedSub,
    CheckedMul,
    CheckedDiv,
    CheckedRem,
    CheckedShl,
    CheckedShr,
    CheckedRotL,
    CheckedRotR,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    // logical
    Or,
    Xor,
    And
}

#[derive(Debug)]
pub struct CheckedNeg;

impl UOpMarker for CheckedNeg {}

#[derive(Debug)]
pub struct Neg;

impl UOpMarker for Neg {}

#[derive(Debug)]
pub struct Not;

impl UOpMarker for Not {}
