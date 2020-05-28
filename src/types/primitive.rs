#![doc(hidden)]

use num_derive::{FromPrimitive, ToPrimitive};
#[repr(u8)]
#[derive(Eq, PartialEq, Copy, Clone, Hash, ToPrimitive, FromPrimitive, Debug)]
pub enum PrimitiveType {
    /// Never type, infallible
    Never = 0,
    U64 = 1,
    U32 = 2,
    U16 = 3,
    U8 = 4,
    I64 = 5,
    I32 = 6,
    I16 = 7,
    I8 = 8,
    F32 = 9,
    F64 = 10,
    Bool = 11,
    Char = 12,

    /// Stack frame
    ///
    /// this type uses metadata for its own purpose
    ///
    /// takes two words - TBD
    ///
    /// ** This type is internal and should not be used for in user code!!! **
    StackFrame = 64,
    /// Return address
    ///
    /// Contains:
    ///  - name of module
    ///  - function name
    ///  - and address of the next instruction
    ///
    /// 3 stack-values wide
    ///
    /// **This type is internal and should not be used for in user code!!!**
    ///
    /// However, it can be used for arbitrary jumps
    ReturnAddr = 65,
    /// Void like type, no value
    Unit = 127,
}

impl PrimitiveType {
    pub fn is_signed(self) -> bool {
        use PrimitiveType::*;
        matches!(self, I8 | I16 | I32 | I64)
    }

    pub fn is_float(self) -> bool {
        use PrimitiveType::*;
        matches!(self, F32 | F64)
    }

    pub fn is_unsigned(self) -> bool {
        use PrimitiveType::*;
        matches!(self, U8 | U16 | U32 | U64)
    }

    pub fn is_bool(self) -> bool {
        matches!(self, PrimitiveType::Bool)
    }

    pub fn is_integer(self) -> bool {
        self.is_signed() || self.is_unsigned()
    }

    pub fn is_number(self) -> bool {
        self.is_signed() || self.is_unsigned() || self.is_float()
    }

    /// Types that are guarantied to occupy one stack value space
    pub fn is_single(self) -> bool {
        use PrimitiveType::*;
        // the never type (!) can never exist as such in does not occupy a stack cells at all
        self.is_number() || matches!(self, Unit | Bool | Char)
    }

    pub fn is_user(self) -> bool {
        self.is_single() || matches!(self, PrimitiveType::Never)
    }

    /// Returns the size of the type in StackData
    pub fn size(self) -> usize {
        if self.is_single() {
            1
        } else if matches!(self, PrimitiveType::Never) {
            0
        } else {
            panic!("No size defined for {:?}", self);
        }
    }
}
impl Default for PrimitiveType {
    fn default() -> Self {
        PrimitiveType::Never
    }
}

/// Gets the vm type from a value
pub trait HasPrimitiveType {
    fn get_type() -> PrimitiveType;
}

macro_rules! impl_has_vm_type {
    ($($t:ty => $vm_type: path),* $(,)?) => {
        $(
            impl HasPrimitiveType for $t {
                fn get_type() -> PrimitiveType {
                    $vm_type
                }
            }
        )*
    };
}

impl_has_vm_type! {
    f64 => PrimitiveType::F64,
    f32 => PrimitiveType::F32,
    char => PrimitiveType::Char,
    bool => PrimitiveType::Bool,
    u64 => PrimitiveType::U64,
    u32 => PrimitiveType::U32,
    u16 => PrimitiveType::U16,
    u8 => PrimitiveType::U8,
    i64 => PrimitiveType::I64,
    i32 => PrimitiveType::I32,
    i16 => PrimitiveType::I16,
    i8 => PrimitiveType::I8,
    () => PrimitiveType::Unit,
}
