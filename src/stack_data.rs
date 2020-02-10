use std::convert::TryInto;
use std::mem::size_of;
/// Data type of the stack data
pub(crate) type StackData = [u8; 8];

/// Data type of one stack cell
pub(crate) type StackBytes = [u8; 16];
/// Convert vm data representation of the type into vm primitive (rust's type representation)
pub(crate) trait ToPrimitive<T> {
    fn to_primitive(&self) -> T;
}

macro_rules! derive_for_types {
    ($($t: ty),*) => {
        $(impl ToPrimitive<$t> for StackData {
            #[inline]
            fn to_primitive(&self) -> $t {
                const S: usize = size_of::<$t>();
                <$t>::from_le_bytes(self[..S].try_into().unwrap())
            }
        })*
    };
}

derive_for_types!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

impl ToPrimitive<bool> for StackData {
    fn to_primitive(&self) -> bool {
        self.iter().any(|&v| v != 0u8)
    }
}

impl ToPrimitive<char> for StackData {
    fn to_primitive(&self) -> char {
        const S: usize = size_of::<u32>();
        std::char::from_u32(u32::from_le_bytes(self[..S].try_into().unwrap()))
            .expect("Invalid char {}")
    }
}
