use super::data::StackData;
use crate::types::Type;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct StackValue {
    /// type of the value
    pub value_type: Type,
    /// Actual data of the value
    pub data: StackData,
}

impl StackValue {
    pub fn new(t: Type, data: StackData) -> Self {
        Self {
            value_type: t,
            data,
        }
    }

    #[allow(dead_code)]
    pub fn default_with_type(t: Type) -> Self {
        Self {
            value_type: t,
            data: Default::default(),
        }
    }
}

impl From<bool> for StackValue {
    fn from(b: bool) -> Self {
        let mut data: [u8; 8] = Default::default();
        if b {
            data[0] |= 0b01;
        }
        StackValue::new(Type::Bool, data)
    }
}
