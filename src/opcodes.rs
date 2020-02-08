use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
pub enum Opcode {
    U64Ld0 = 0,
    I64Ld0 = 1,
    LdTyped0 = 2,
    LdType = 3,
    UAdd = 10,
    IAdd = 11,
    FAdd = 12,
    HWide = 255,
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::fmt::{self, Write};

    use pretty_assertions::{assert_eq, assert_ne};

    use crate::decoder::handlers::functions::noop as d_noop;
    use crate::decoder::HANDLERS as D_HANDLERS;
    use crate::interpreter::handlers::functions::noop as i_noop;
    use crate::interpreter::HANDLERS as I_HANDLERS;

    use super::*;

    /// Check if all opcodes have handlers and that they don't noop
    #[test]
    fn all_opcodes_have_handlers() -> fmt::Result {
        let mut invalid_decoders = Vec::new();
        let mut invalid_interpreters = Vec::new();
        for i in 0u8..=255 {
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
