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
        let data_0 = stack.get(0);
        let value_type = &self.1.value_type;
        let alt = f.alternate();
        let mut s = f.debug_struct("stack_value");
        if alt {
            match value_type {
                VmType::Primitive(p) => {
                    s.field("type", p);
                    let data_0 = *data_0.unwrap();
                    match p {
                        PrimitiveType::StackFrame => todo!(),
                        PrimitiveType::ReturnAddr => todo!(),
                        PrimitiveType::Unit => s.field("data", &"(unit)"),
                        PrimitiveType::Never => s.field("data", &"(never!)"),
                        PrimitiveType::U64 => s.field("data", &u64::from_single(data_0)),
                        PrimitiveType::I64 => s.field("data", &i64::from_single(data_0)),
                        PrimitiveType::U32 => s.field("data", &u32::from_single(data_0)),
                        PrimitiveType::I32 => s.field("data", &i32::from_single(data_0)),
                        PrimitiveType::U16 => s.field("data", &u16::from_single(data_0)),
                        PrimitiveType::I16 => s.field("data", &i16::from_single(data_0)),
                        PrimitiveType::U8 => s.field("data", &u8::from_single(data_0)),
                        PrimitiveType::I8 => s.field("data", &i8::from_single(data_0)),
                        PrimitiveType::F32 => s.field("data", &f32::from_single(data_0)),
                        PrimitiveType::F64 => s.field("data", &f64::from_single(data_0)),
                        PrimitiveType::Bool => s.field("data", &bool::from_single(data_0)),
                        PrimitiveType::Char => s.field("data", &char::from_single(data_0)),
                        PrimitiveType::SStr => {
                            let ptr = usize::from_single(data_0) as *const u8;
                            let len: usize = self.0.get(1).ok_or(fmt::Error)?.into_primitive();
                            let str = unsafe { from_utf8_unchecked(from_raw_parts(ptr, len)) };
                            s.field("data", &str)
                        }
                    };
                }
                VmType::PointedType(p) => {
                    match p.deref() {
                        PointedType::SArr(a) => {
                            s.field("data", &"<S Arr>");
                            s.field("type", &format!("[{:?};{}]", a.pointer, a.len));
                        }
                        PointedType::Ref(r) => {
                            // TODO: add cycle to the type of ref
                            s.field("data", &usize::from_single(*data_0.unwrap()));
                            s.field("location", &r.points_to);
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
