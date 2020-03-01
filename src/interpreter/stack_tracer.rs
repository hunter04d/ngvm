use std::fmt::{self, Debug, Formatter};

use crate::stack::data::{FromSingle, StackData};
use crate::stack::metadata::StackMetadata;
use crate::types::Type;

/// Traces the stack value contained starting from slice
pub struct StackTracer<'a>(pub &'a [StackData], pub &'a StackMetadata);

impl<'a> Debug for StackTracer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let stack = self.0;
        let data_0 = stack[0];
        let value_type = self.1.value_type;
        let alt = f.alternate();
        let mut s = f.debug_struct("stack_value");
        s.field("type", &value_type);
        if alt {
            let repr: Box<dyn Debug> = match value_type {
                Type::StackFrame => todo!(),
                Type::ReturnAddr => todo!(),
                Type::Unit => Box::new("(unit)"),
                Type::Never => Box::new("(never!)"),
                Type::Pointed => todo!(),
                Type::U64 => Box::new(u64::from_single(data_0)),
                Type::I64 => Box::new(i64::from_single(data_0)),
                Type::U32 => Box::new(u32::from_single(data_0)),
                Type::I32 => Box::new(i32::from_single(data_0)),
                Type::U16 => Box::new(u16::from_single(data_0)),
                Type::I16 => Box::new(i16::from_single(data_0)),
                Type::U8 => Box::new(u8::from_single(data_0)),
                Type::I8 => Box::new(i8::from_single(data_0)),
                Type::F32 => Box::new(f32::from_single(data_0)),
                Type::F64 => Box::new(f64::from_single(data_0)),
                Type::Bool => Box::new(bool::from_single(data_0)),
                Type::Char => Box::new(char::from_single(data_0)),
            };
            s.field("data", &repr);
        } else {
            s.field("data", &data_0);
        }
        s.finish()
    }
}
