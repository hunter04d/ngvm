use crate::stack_data::{StackBytes, StackData};
use crate::types::Type;
use std::convert::{TryFrom, TryInto};

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct StackValue {
    /// type of the value
    pub value_type: Type,
    /// Reserved for future use
    pub reserved: [u8; 7],
    /// Actual data of the value
    pub data: StackData,
}

impl StackValue {
    pub fn new(t: Type, data: StackData) -> Self {
        Self {
            value_type: t,
            reserved: Default::default(),
            data,
        }
    }

    pub fn default_with_type(t: Type) -> Self {
        Self {
            value_type: t,
            reserved: Default::default(),
            data: Default::default(),
        }
    }

    pub fn from_bytes(bytes: StackBytes) -> Result<Self, num_enum::TryFromPrimitiveError<Type>> {
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

impl TryFrom<StackBytes> for StackValue {
    type Error = num_enum::TryFromPrimitiveError<Type>;

    fn try_from(value: [u8; 16]) -> Result<Self, Self::Error> {
        Self::from_bytes(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;
    use crate::stack_data::StackBytes;
    use std::mem::size_of;

    #[test]
    fn stack_value_should_be_16_bytes() {
        assert_eq!(size_of::<StackValue>(), size_of::<StackBytes>());
    }
}
