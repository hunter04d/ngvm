use std::fmt::{self, Debug, Formatter};

use crate::meta::StackMeta;
use crate::stack::data::{FromSingle, IntoPrimitive, StackData};
use crate::types::{PointedType, PrimitiveType, VmType};
use std::ops::Deref;
use std::slice::from_raw_parts;
use std::str::from_utf8_unchecked;

/// Traces the stack value contained in a slice of stack data
pub struct StackTracer<'a>(pub &'a [StackData], pub &'a StackMeta);

impl<'a> Debug for StackTracer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: DST tracing
        let stack = self.0;
        let data_0 = stack.get(0).copied().ok_or(fmt::Error);
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
                        PrimitiveType::U64 => Box::new(u64::from_single(data_0?)),
                        PrimitiveType::I64 => Box::new(i64::from_single(data_0?)),
                        PrimitiveType::U32 => Box::new(u32::from_single(data_0?)),
                        PrimitiveType::I32 => Box::new(i32::from_single(data_0?)),
                        PrimitiveType::U16 => Box::new(u16::from_single(data_0?)),
                        PrimitiveType::I16 => Box::new(i16::from_single(data_0?)),
                        PrimitiveType::U8 => Box::new(u8::from_single(data_0?)),
                        PrimitiveType::I8 => Box::new(i8::from_single(data_0?)),
                        PrimitiveType::F32 => Box::new(f32::from_single(data_0?)),
                        PrimitiveType::F64 => Box::new(f64::from_single(data_0?)),
                        PrimitiveType::Bool => Box::new(bool::from_single(data_0?)),
                        PrimitiveType::Char => Box::new(char::from_single(data_0?)),
                        PrimitiveType::SStr => {
                            let ptr = usize::from_single(data_0?) as *const u8;
                            let len: usize = self.0.get(1).ok_or(fmt::Error)?.into_primitive();
                            let str = unsafe { from_utf8_unchecked(from_raw_parts(ptr, len)) };
                            Box::new(str)
                        }
                    };
                    s.field("data", &repr);
                    s.field("type", &value_type);
                }
                VmType::PointedType(p) => {
                    match p.deref() {
                        PointedType::SArr(a) => {
                            s.field("data", &"<S Arr>");
                            s.field("type", &format!("[{:?};{}]", a.pointer, a.len));
                        }
                        PointedType::Ref(r) => {
                            // TODO: add cycle to the type of ref
                            s.field("data", &usize::from_single(data_0?));
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
