use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Eq, PartialEq, Copy, Clone, Hash, IntoPrimitive, TryFromPrimitive)]
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
