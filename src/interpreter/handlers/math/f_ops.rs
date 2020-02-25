use std::convert::TryInto;
use std::mem::size_of;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use crate::code::Chunk;
use crate::opcodes::refs;
use crate::types::Type;
use crate::stack_data::FromSingle;
use crate::Vm;

macro_rules! handle_f_op {
    ($chunk: ident, $vm: ident, $op: ident) => {{
        let res_ref = $chunk.read_ref(0);
        let op1 = &$vm.stack[$chunk.read_ref(1)];
        let op2 = &$vm.stack[$chunk.read_ref(2)];
        let res = &$vm.stack[res_ref];
        if res.value_type == op1.value_type && op1.value_type == op2.value_type {
            match res.value_type {
                Type::F64 => {
                    let op1 = f64::from_single(op1.data);
                    let op2 = f64::from_single(op2.data);
                    let r = op1.$op(op2);
                    $vm.stack_value_mut(res_ref).data = r.to_le_bytes();
                }
                Type::F32 => {
                    const S: usize = size_of::<f32>();
                    let op1 = f32::from_single(op1.data);
                    let op2 = f32::from_single(op2.data);
                    let r = op1.$op(op2);
                    $vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
                }
                _ => panic!("Type mismatch"),
            }
        } else {
            panic!("Type mismatch");
        }
        1 + refs(3)
    }};
}

macro_rules! handle_f_ops {
   ($($fn_name: ident => $method_name: ident),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_f_op!(chunk, vm, $method_name)
        })*

    };
}

handle_f_ops! {
    handle_f_add => add,
    handle_f_sub => sub,
    handle_f_mul => mul,
    handle_f_div => div,
    handle_f_rem => rem
}

pub(in crate::interpreter) fn handle_f_neg(chunk: &Chunk, vm: &mut Vm) -> usize {
    let res_ref = chunk.read_ref(0);
    let op_ref = chunk.read_ref(1);
    let op = vm.stack_value(op_ref);
    let res = vm.stack_value(res_ref);
    if op.value_type == res.value_type {
        match res.value_type {
            Type::F64 => {
                let op: f64 = f64::from_le_bytes(op.data);
                vm.stack_value_mut(res_ref).data = f64::to_le_bytes(op.neg());
            }
            Type::F32 => {
                const S: usize = size_of::<f32>();
                let r = -f32::from_le_bytes(op.data[..S].try_into().unwrap());
                vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
            }
            _ => panic!("Type mismatch"),
        }
    } else {
        panic!("Type mismatch")
    }
    1 + refs(2)
}
