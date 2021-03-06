use std::convert::TryInto;
use std::mem::size_of;

use crate::vm::ValueLocation;

/// Data type of the stack data
pub(crate) type StackData = [u8; 8];

pub(crate) trait FromSingle<T> {
    fn from_single(obj: T) -> Self;
}

pub(crate) trait FromDouble<T> {
    fn from_double(obj: T) -> Self;
}

pub trait FromPrimitive<T> {
    fn from_primitive(obj: T) -> Self;
}

/// Convert vm data representation of the type into vm primitive (rust's type representation)
pub(crate) trait IntoPrimitive<T> {
    fn into_primitive(self) -> T;
}

pub trait IntoStackData {
    fn into_stack_data(self) -> StackData;
}

macro_rules! derive_from_single_for_types {
    ($($t: ty),*) => {
        $(impl FromSingle<StackData> for $t {
            #[inline]
            fn from_single(obj: StackData) -> Self {
                const S: usize = size_of::<$t>();
                <$t>::from_le_bytes(obj[..S].try_into().unwrap())
            }
        })*
    };
}

derive_from_single_for_types!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, usize);

impl FromSingle<StackData> for bool {
    #[inline]
    fn from_single(obj: StackData) -> Self {
        obj.iter().any(|&v| v != 0u8)
    }
}

impl FromSingle<StackData> for char {
    fn from_single(obj: [u8; 8]) -> Self {
        const S: usize = size_of::<u32>();
        std::char::from_u32(u32::from_le_bytes(obj[..S].try_into().unwrap()))
            .expect("Invalid char {}")
    }
}

impl FromPrimitive<ValueLocation> for StackData {
    fn from_primitive(obj: ValueLocation) -> Self {
        match obj {
            ValueLocation::Stack(sr) => StackData::from_primitive(sr),
            ValueLocation::Heap(hr) => StackData::from_primitive(hr as usize),
        }
    }
}

/// Generic impl of this type
impl<T: FromSingle<StackData>> IntoPrimitive<T> for StackData {
    fn into_primitive(self) -> T {
        T::from_single(self)
    }
}

macro_rules! derive_from_primitive_for_types {
    ($($t: ty),*) => {
        $(impl FromPrimitive<$t> for StackData {
            fn from_primitive(obj: $t) -> Self {
                const S: usize = size_of::<$t>();
                let mut res = StackData::default();
                res[..S].copy_from_slice(&obj.to_le_bytes());
                res
            }
        })*
    };
}

derive_from_primitive_for_types!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, usize);

impl<T> IntoStackData for T
where
    StackData: FromPrimitive<T>,
{
    fn into_stack_data(self) -> StackData {
        StackData::from_primitive(self)
    }
}

impl IntoStackData for StackData {
    fn into_stack_data(self) -> StackData {
        self
    }
}

impl FromPrimitive<bool> for StackData {
    fn from_primitive(obj: bool) -> Self {
        let mut data: StackData = Default::default();
        if obj {
            data[0] |= 0b01;
        }
        data
    }
}

impl FromPrimitive<()> for StackData {
    fn from_primitive(_: ()) -> Self {
        Default::default()
    }
}

impl FromPrimitive<char> for StackData {
    fn from_primitive(obj: char) -> Self {
        u32::from(obj).into_stack_data()
    }
}
