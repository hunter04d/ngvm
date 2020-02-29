use crate::code::Chunk;
use crate::opcodes::refs;
use crate::stack_data::{FromSingle, IntoStackData};
use crate::types::Type;
use crate::Vm;

macro_rules! handle_u_op {
    ($chunk: ident, $vm: ident, $op: ident) => {{
        let res_ref = $chunk.read_ref(0);
        let op1 = $vm.stack_value($chunk.read_ref(1));
        let op2 = $vm.stack_value($chunk.read_ref(2));
        let res = $vm.stack_value(res_ref);
        if res.value_type == op1.value_type && op1.value_type == op2.value_type {
            $vm.stack_value_mut(res_ref).data = match res.value_type {
                Type::U64 => {
                    let op1 = u64::from_single(op1.data);
                    let op2 = u64::from_single(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    r.into_stack_data()
                }
                Type::U32 => {
                    let op1 = u32::from_single(op1.data);
                    let op2 = u32::from_single(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    r.into_stack_data()
                }
                Type::U16 => {
                    let op1 = u16::from_single(op1.data);
                    let op2 = u16::from_single(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    r.into_stack_data()
                }
                Type::U8 => {
                    let op1 = u8::from_single(op1.data);
                    let op2 = u8::from_single(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    r.into_stack_data()
                }
                _ => panic!("Type mismatch"),
            }
        } else {
            panic!("Type mismatch");
        }
        1 + refs(3)
    }};
}

macro_rules! handle_u_ops {
   ($($fn_name: ident => $method_name: ident),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_u_op!(chunk, vm, $method_name)
        })*

    };
}

handle_u_ops! {
    handle_u_add => checked_add,
    handle_u_sub => checked_sub,
    handle_u_mul => checked_mul,
    handle_u_div => checked_div,
    handle_u_rem => checked_rem
}
