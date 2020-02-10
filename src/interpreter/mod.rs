use std::fmt::{self, Debug, Formatter};

pub(crate) use handlers::HANDLERS;

use crate::stack_data::ToPrimitive;
use crate::types::Type;
use crate::StackValue;

pub mod handlers;

/// Traces the stack value contained starting from slice
pub struct StackTracer<'a>(pub &'a [StackValue]);

impl<'a> Debug for StackTracer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let stack = self.0;
        let cell_0 = &stack[0];
        let data_0 = cell_0.data;
        let alt = f.alternate();
        let mut s = f.debug_struct("stack_value");
        s.field("type", &cell_0.value_type);
        if alt {
            let repr: Box<dyn Debug> = match cell_0.value_type {
                Type::StackFrame => todo!(),
                Type::ReturnAddr => todo!(),
                Type::Unit => Box::new("(unit)"),
                Type::Never => Box::new("(never!)"),
                Type::Pointed => todo!(),
                Type::U64 => {
                    let v: u64 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::I64 => {
                    let v: i64 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::U32 => {
                    let v: u32 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::I32 => {
                    let v: i32 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::U16 => {
                    let v: u16 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::I16 => {
                    let v: i16 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::U8 => {
                    let v: u8 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::I8 => {
                    let v: i8 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::F32 => {
                    let v: f32 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::F64 => {
                    let v: f64 = data_0.to_primitive();
                    Box::new(v)
                }
                Type::Bool => {
                    let v: bool = data_0.to_primitive();
                    Box::new(v)
                }
                Type::Char => {
                    let v: bool = data_0.to_primitive();
                    Box::new(v)
                }
            };
            s.field("data", &repr);
        } else {
            s.field("data", &cell_0.data);
        }
        s.finish()
    }
}
