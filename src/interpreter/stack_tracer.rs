use std::fmt::{self, Debug, Formatter};

use crate::meta::StackMeta;
use crate::stack::data::{FromSingle, StackData};
use crate::types::{PointedType, PrimitiveType, VmType};
use std::ops::Deref;

/// Traces the stack value contained in a slice of stack data
pub struct StackTracer<'a>(pub &'a [StackData], pub &'a StackMeta);

impl<'a> Debug for StackTracer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: DST tracing
        let stack = self.0;
        let data_0 = stack.get(0).copied();
        let value_type = &self.1.value_type;
        let alt = f.alternate();
        let mut s = f.debug_struct("stack_value");
        if alt {
            match value_type {
                VmType::Primitive(p) => {
                    let repr: Box<dyn Debug> = match p {
                        PrimitiveType::StackFrame => todo!(),
                        PrimitiveType::ReturnAddr => todo!(),
                        PrimitiveType::Unit => Box::new("(unit)"),
                        PrimitiveType::Never => Box::new("(never!)"),
                        PrimitiveType::U64 => Box::new(u64::from_single(data_0.unwrap())),
                        PrimitiveType::I64 => Box::new(i64::from_single(data_0.unwrap())),
                        PrimitiveType::U32 => Box::new(u32::from_single(data_0.unwrap())),
                        PrimitiveType::I32 => Box::new(i32::from_single(data_0.unwrap())),
                        PrimitiveType::U16 => Box::new(u16::from_single(data_0.unwrap())),
                        PrimitiveType::I16 => Box::new(i16::from_single(data_0.unwrap())),
                        PrimitiveType::U8 => Box::new(u8::from_single(data_0.unwrap())),
                        PrimitiveType::I8 => Box::new(i8::from_single(data_0.unwrap())),
                        PrimitiveType::F32 => Box::new(f32::from_single(data_0.unwrap())),
                        PrimitiveType::F64 => Box::new(f64::from_single(data_0.unwrap())),
                        PrimitiveType::Bool => Box::new(bool::from_single(data_0.unwrap())),
                        PrimitiveType::Char => Box::new(char::from_single(data_0.unwrap())),
                    };
                    s.field("data", &repr);
                    s.field("type", &value_type);
                }
                VmType::PointedType(p) => {
                    match p.deref() {
                        PointedType::Arr { .. } => unimplemented!("Arrays are not implemented"),
                        PointedType::Ref(r) => {
                            // TODO: add cycle to the type of ref
                            s.field("data", &usize::from_single(data_0.unwrap()));
                            s.field("type", &format!("{}", r));
                        }
                    }
                }
            }
        } else {
            s.field("data", &data_0);
        }
        s.finish()
    }
}
