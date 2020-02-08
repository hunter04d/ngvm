use std::convert::TryInto;

use crate::types::Type;

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct StackValue {
    /// type of the value
    pub value_type: Type,
    /// Reserved for future use
    pub reserved: [u8; 7],
    /// Actual data of the value
    pub data: [u8; 8],
}

impl StackValue {
    pub fn new(t: Type, data: [u8; 8]) -> Self {
        Self {
            value_type: t,
            reserved: Default::default(),
            data,
        }
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Result<Self, num_enum::TryFromPrimitiveError<Type>> {
        let value_type: Type = bytes[0].try_into()?;
        let reserved: [u8; 7] = Default::default();
        let data = bytes[8..].try_into().unwrap();
        Ok(Self {
            value_type,
            reserved,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::mem::align_of;

    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    #[test]
    fn stack_value_should_be_16_bytes() {
        assert_eq!(align_of::<StackValue>(), 16);
    }
}
