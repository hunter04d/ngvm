use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Eq, PartialEq, Copy, Clone, Hash, IntoPrimitive, TryFromPrimitive, Debug)]
pub enum Type {
    U64 = 0,
    I64 = 1,
    U32 = 2,
    I32 = 3,
    U16 = 4,
    I16 = 5,
    U8 = 6,
    I8 = 7,
    F32 = 8,
    F64 = 9,
    Bool = 10,
    Char = 11,
    /// Stack frame
    ///
    /// this type uses metadata for its own purpose
    ///
    /// takes one word
    ///
    /// ** This type is internal and should not be used for in user code!!! **
    StackFrame = 64,
    /// Return address
    ///
    /// Contains:
    ///  - name of module
    ///  - function index
    ///  - and address of the next instruction
    ///
    /// 2 stack-values wide
    ///
    /// **This type is internal and should not be used for in user code!!!**
    ///
    /// However, it can be used for arbitrary jumps
    ReturnAddr = 65,
    /// Void like type, no value
    Unit = 127,
    /// Never type, infallible
    Never = 128,
    /// Points to the type in the constant pool
    ///
    /// 2 stack-values wide
    Pointed = 0xFF,
}

impl Type {
    pub fn is_signed(self) -> bool {
        use Type::*;
        match self {
            I8 | I16 | I32 | I64 => true,
            _ => false,
        }
    }

    pub fn is_float(self) -> bool {
        use Type::*;
        match self {
            F32 | F64 => true,
            _ => false,
        }
    }

    pub fn is_unsigned(self) -> bool {
        use Type::*;
        match self {
            U8 | U16 | U32 | U64 => true,
            _ => false,
        }
    }

    pub fn is_bool(self) -> bool {
        if let Type::Bool = self {
            true
        } else {
            false
        }
    }

    pub fn is_number(self) -> bool {
        self.is_signed() || self.is_unsigned() || self.is_float()
    }

    /// Types that are guarantied to occupy one stack value space
    pub fn is_single(self) -> bool {
        use Type::*;
        if self.is_number() {
            true
        } else {
            match self {
                // never can never exist as such in does not occupy a stack cells at all
                Unit | Bool | Char => true,
                _ => false,
            }
        }
    }
}

#[allow(unused_macros)]
macro_rules! single_type_map {
    () => {
        Type::U64 => u64,
        Type::I64 => i64,
        Type::U32 => u32,
        Type::I32 => i32,
        Type::U16 => u16,
        Type::I16 => i16,
        Type::U8 => u8,
        Type::I8 => i8,
        Type::F64 => f64,
        Type::F32 => f32
    };
}
