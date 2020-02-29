use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use crate::code::Chunk;
use crate::opcodes::refs;
use crate::stack_data::{FromSingle, IntoStackData};
use crate::types::Type;
use crate::Vm;

macro_rules! handle_f_op {
    ($chunk: ident, $vm: ident, $op: ident) => {{
        let res_ref = $chunk.read_ref(0);
        let op1 = &$vm.stack[$chunk.read_ref(1)];
        let op2 = &$vm.stack[$chunk.read_ref(2)];
        let res = &$vm.stack[res_ref];
        if res.value_type == op1.value_type && op1.value_type == op2.value_type {
             $vm.stack_value_mut(res_ref).data = match res.value_type {
                Type::F64 => {
                    let op1 = f64::from_single(op1.data);
                    let op2 = f64::from_single(op2.data);
                    let r = op1.$op(op2);
                    r.into_stack_data()
                }
                Type::F32 => {
                    let op1 = f32::from_single(op1.data);
                    let op2 = f32::from_single(op2.data);
                    let r = op1.$op(op2);
                    r.into_stack_data()
                }
                _ => unreachable!("Types mismatch"),
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
        vm.stack_value_mut(res_ref).data = match res.value_type {
            Type::F64 => f64::from_single(op.data).neg().into_stack_data(),
            Type::F32 => f32::from_single(op.data).neg().into_stack_data(),
            _ => unreachable!("Types mismatch"),
        }
    } else {
        panic!("Type mismatch")
    }
    1 + refs(2)
}
