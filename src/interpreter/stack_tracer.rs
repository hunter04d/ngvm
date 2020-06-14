use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;
use std::slice::from_raw_parts;
use std::str::from_utf8_unchecked;

use crate::meta::StackMeta;
use crate::stack::data::{FromSingle, IntoPrimitive, StackData};
use crate::types::{PointedType, PrimitiveType, VmType};

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
                            if let Some(p) = a.pointer.primitive() {
                                let vec = self
                                    .0
                                    .iter()
                                    .map(|v| display_of_primitive(p, *v))
                                    .collect::<Vec<_>>();
                                s.field("data", &vec);
                            } else {
                                s.field("data", &"<S Arr>");
                            }
                            // TODO: better display for arrays

                            s.field("type", &format!("[{:?};{}]", a.pointer, a.len));
                        }
                        PointedType::Ref(r) => {
                            // TODO: add cycle to the type of ref
                            s.field("data", &usize::from_single(*data_0.unwrap()));
                            s.field("location", &r.points_to);
                            s.field("type", &format!("{}", r));
                        }
                        PointedType::Boxed(t) => {
                            let ptr = usize::from_single(*data_0.unwrap()) as *const ();
                            s.field("data", &ptr);
                            s.field("type", &format!("Box<{:?}>", t));
                        }
                    }
                }
            }
        } else {
            let data = self
                .0
                .iter()
                .flatten()
                .map(|v| format!("{:02x}", v))
                .collect::<Vec<_>>()
                .join("");
            s.field("data", &data);
        }
        s.finish()
    }
}

fn display_of_primitive(p: PrimitiveType, data: StackData) -> Box<dyn Debug> {
    match p {
        PrimitiveType::Never => Box::new("(never!)"),
        PrimitiveType::U64 => Box::new(u64::from_single(data)),
        PrimitiveType::U32 => Box::new(u32::from_single(data)),
        PrimitiveType::U16 => Box::new(u16::from_single(data)),
        PrimitiveType::U8 => Box::new(u8::from_single(data)),
        PrimitiveType::I64 => Box::new(i64::from_single(data)),
        PrimitiveType::I32 => Box::new(i32::from_single(data)),
        PrimitiveType::I16 => Box::new(i16::from_single(data)),
        PrimitiveType::I8 => Box::new(i8::from_single(data)),
        PrimitiveType::F32 => Box::new(f32::from_single(data)),
        PrimitiveType::F64 => Box::new(f64::from_single(data)),
        PrimitiveType::Bool => Box::new(bool::from_single(data)),
        PrimitiveType::Char => Box::new(char::from_single(data)),
        PrimitiveType::SStr => todo!(),
        PrimitiveType::StackFrame => unimplemented!(),
        PrimitiveType::ReturnAddr => unimplemented!(),
        PrimitiveType::Unit => Box::new("(unit)"),
    }
}
