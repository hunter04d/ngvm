use std::mem::size_of;

use crate::types::PrimitiveType;
use std::convert::TryInto;
#[derive(Debug)]
pub enum Constant {
    Value([u8; 16]),
    String(Box<str>),
    Type(PrimitiveType),
    // TODO
    /// Pointed type for things like Arr->i32, Ref->u64, etc.
    PointedType,
}

macro_rules! impl_from {
   ( $($t: ty),* )=> {
       $( impl From<$t> for Constant {
            fn from(v: $t) -> Self {
                let bytes = v.to_le_bytes();
                let mut res = [0u8; 16];
                res[..size_of::<$t>()].copy_from_slice(&bytes);
                Constant::Value(res)
            }
        })*
   };
}

impl_from!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl From<PrimitiveType> for Constant {
    fn from(t: PrimitiveType) -> Self {
        Constant::Type(t)
    }
}

/// Pool of constants of specific module
///
/// Essentially new-type around `Vec<`[`Constant`](self::Constant)`>`
#[derive(Debug)]
pub struct ConstantPool(Vec<Constant>);

impl ConstantPool {
    pub fn new(constants: Vec<Constant>) -> Self {
        Self(constants)
    }

    pub fn get(&self, index: usize) -> Option<&Constant> {
        self.0.get(index)
    }

    pub fn get_type(&self, index: usize) -> Option<PrimitiveType> {
        if let Some(Constant::Type(t)) = self.get(index) {
            Some(*t)
        } else {
            None
        }
    }

    pub fn get_single(&self, index: usize) -> Option<[u8; 8]> {
        if let Some(Constant::Value(v)) = self.get(index) {
            Some(v[..8].try_into().unwrap())
        } else {
            None
        }
    }

    pub fn get_wide(&self, index: usize) -> Option<[u8; 16]> {
        if let Some(Constant::Value(v)) = self.get(index) {
            Some(*v)
        } else {
            None
        }
    }
}
