use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u16)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
pub enum Opcode {
    U64Ld0 = 0,
    I64Ld0 = 1,
    LdTyped0 = 2,
    LdType = 3,
    LdUnit = 4,
    LdTrue = 5,
    LdFalse = 6,
    LdSS = 8,
    LdDS = 9,
    // u ops
    UAdd = 10,
    USub = 11,
    UMul = 12,
    UDiv = 13,
    URem = 14,
    // i ops
    IAdd = 15,
    ISub = 16,
    IMul = 17,
    IDiv = 18,
    IRem = 19,
    INeg = 20,

    // f ops
    FAdd = 21,
    FSub = 22,
    FMul = 23,
    FDiv = 24,
    FRem = 25,
    FNeg = 26,

    // Todo: bool ops
    // the behaviour of those ops are type independent,
    // assuming the type is a primitive
    BAnd = 27,
    BOr = 28,
    BNot = 29,
    BBe = 30,
    BXor = 31,
    LAnd = 32,
    LOr = 33,
    LNot = 34,
    LXor = 35,

    // Todo: shifts,
    Shl = 36,
    Shr = 37,
    RotL = 38,
    RotR = 39,
    // TODO: comparisons
    Ge = 40,
    Gt = 41,
    Le = 42,
    Lt = 43,
    Eq = 44,
    Ne = 45,
    // TODO: jump
    /// Jump <offset>
    J = 46,
    /// Jump <condition> <offset>
    JC = 47,
    StartScope = 48,
    EndScope = 49,
    // TODO: call
    Call = 50, // Call <POOL REF>
    Ret = 51, // Return
    IsType = 52,
    // TODO: arrays if have time

    //
    TraceStackValue = 254,
    /// Handle wide, not an actually  a valid value for opcode
    HWide = 255,
}


pub enum OpcodeKind {
    Single(u8),
    Double(u8)
}

impl Opcode {
    pub fn kind(self) -> OpcodeKind {
        let num: u16 = self.into();
        if num < 256 {
            OpcodeKind::Single(num as u8)
        } else {
            OpcodeKind::Double((num - 256) as u8)
        }
    }

    pub fn bytes(self) -> Vec<u8> {
        match self.kind() {
            OpcodeKind::Single(c) => vec![c],
            OpcodeKind::Double(c) => vec![255, c],
        }
    }

    pub fn size(self) -> usize {
        match self.kind() {
            OpcodeKind::Single(_) => 1,
            OpcodeKind::Double(_) => 2,
        }
    }
}
