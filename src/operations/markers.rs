//! Markers for the operations.

use super::UOpMarker;
macro_rules! gen_markers {
    ($($t: ident),*) => {
        $(
        pub struct $t;

        impl BiOpMarker for $t {}
        )*
    };
}

gen_markers! {
    CheckedAdd,
    CheckedSub,
    CheckedMul,
    CheckedDiv,
    CheckedRem,
    CheckedShl,
    CheckedShr,
    Eq,
    NEq,
    Lt,
    Gt,
    Le,
    Ge
}

pub struct CheckedNeg;

impl UOpMarker for CheckedNeg {}
