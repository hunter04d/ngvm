use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
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
    // the behaviour of those ops are type independent assuming the type is a primitive
    BAnd = 27,
    BOr = 28,
    BNot = 29,
    BBe = 30,
    BXor = 31,
    // Todo: Logical ops
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

    TraceStackValue = 254,
    HWide = 255,
}

/// Type of the reference to a value in bytecode
pub type Ref = usize;


#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::fmt::{self, Write};

    use crate::decoder::handlers::noop as d_noop;
    use crate::decoder::HANDLERS as D_HANDLERS;
    use crate::interpreter::handlers::noop as i_noop;
    use crate::interpreter::HANDLERS as I_HANDLERS;

    use super::*;

    /// Check if all opcodes have handlers and that they don't noop
    #[test]
    fn all_opcodes_have_handlers() -> fmt::Result {
        let mut invalid_decoders = Vec::new();
        let mut invalid_interpreters = Vec::new();
        for i in std::u8::MIN..=std::u8::MAX {
            let op_result = Opcode::try_from(i);
            if let Ok(op) = op_result {
                if I_HANDLERS[i as usize] as *const () == i_noop as *const () {
                    invalid_interpreters.push(op);
                }
                if D_HANDLERS[i as usize] as *const () == d_noop as *const () {
                    invalid_decoders.push(op);
                }
            }
        }
        let mut panic_msg = String::new();
        if !invalid_interpreters.is_empty() {
            writeln!(
                &mut panic_msg,
                "{:?} opcodes don't have valid **interpret** handlers",
                invalid_interpreters
            )?;
        }
        if !invalid_decoders.is_empty() {
            writeln!(
                &mut panic_msg,
                "{:?} opcodes don't have valid **decode** handlers",
                invalid_decoders
            )?;
        }
        if !panic_msg.is_empty() {
            panic!(panic_msg)
        }
        Ok(())
    }
}
