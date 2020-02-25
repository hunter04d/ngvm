use std::mem::size_of;
use std::ops::Neg;

use crate::code::Chunk;
use crate::opcodes::refs;
use crate::types::Type;
use crate::stack_data::FromSingle;
use crate::Vm;

macro_rules! handle_i_op {
    ($chunk: ident, $vm: ident, $op: ident) => {{
        let res_ref = $chunk.read_ref(0);
        let op1 = $vm.stack_value($chunk.read_ref(1));
        let op2 = $vm.stack_value($chunk.read_ref(2));
        let res = $vm.stack_value(res_ref);
        if res.value_type == op1.value_type && op1.value_type == op2.value_type {
            match res.value_type {
                Type::I64 => {
                    let op1 = i64::from_single(op1.data);
                    let op2 = i64::from_single(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack_value_mut(res_ref).data = r.to_le_bytes();
                }
                Type::I32 => {
                    const S: usize = size_of::<i32>();
                    let op1 = i32::from_single(op1.data);
                    let op2 = i32::from_single(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
                }
                Type::I16 => {
                    const S: usize = size_of::<i16>();
                    let op1 = i16::from_single(op1.data);
                    let op2 = i16::from_single(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack[res_ref].data[..S].copy_from_slice(&r.to_le_bytes());
                }
                Type::I8 => {
                    const S: usize = size_of::<i8>();
                    let op1 = i8::from_single(op1.data);
                    let op2 = i8::from_single(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack[res_ref].data[..S].copy_from_slice(&r.to_le_bytes());
                }
                _ => panic!("Type mismatch"),
            }
        } else {
            panic!("Type mismatch");
        }
        1 + refs(3)
    }};
}

macro_rules! handle_i_ops {
   ($($fn_name: ident => $method_name: ident),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_i_op!(chunk, vm, $method_name)
        })*

    };
}

handle_i_ops! {
    handle_i_add => checked_add,
    handle_i_sub => checked_sub,
    handle_i_mul => checked_mul,
    handle_i_div => checked_div,
    handle_i_rem => checked_rem
}


pub(in crate::interpreter) fn handle_i_neg(chunk: &Chunk, vm: &mut Vm) -> usize {
    let res_ref = chunk.read_ref(0);
    let op_ref = chunk.read_ref(1);
    let op = vm.stack_value(op_ref);
    let res = vm.stack_value(res_ref);
    if op.value_type == res.value_type {
        match res.value_type {
            Type::I64 => {
                let op: i64 = i64::from_single(op.data);
                vm.stack_value_mut(res_ref).data = i64::to_le_bytes(op.neg());
            }
            Type::I32 => {
                const S: usize = size_of::<i32>();
                let r = -i32::from_single(op.data);
                vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
            }
            Type::I16 => {
                const S: usize = size_of::<i16>();
                let r = -i16::from_single(op.data);
                vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
            }
            Type::I8 => {
                const S: usize = size_of::<i8>();
                let r = -i8::from_single(op.data);
                vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
            }
            _ => panic!("Type mismatch"),
        }
    } else {
        panic!("Type mismatch")
    }

    1 + refs(2)
}
