use std::convert::TryInto;
use std::mem::size_of;
use std::ops::{Add, Div, Mul, Sub};

use crate::types::Type;
use crate::code::Chunk;
use crate::{Vm, StackValue};

pub(super) fn handle_u64_ld0(_: &Chunk, vm: &mut Vm) -> usize {
    vm.stack
        .push(StackValue::new(Type::U64, Default::default()));
    1
}

pub(super) fn handle_i64_ld0(_: &Chunk, vm: &mut Vm) -> usize {
    vm.stack
        .push(StackValue::new(Type::I64, Default::default()));
    1
}

pub(super) fn handle_ld_typed0(chunk: &Chunk, vm: &mut Vm) -> usize {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref(0);
    let t = pool.get_type_or_panic(type_ref);
    vm.stack.push(StackValue::new(t, Default::default()));
    1 + refs(1)
}
pub(super) fn handle_ld_type(chunk: &Chunk, vm: &mut Vm) -> usize {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref(0);
    let value_ref = chunk.read_ref(1);
    let t = pool.get_type_or_panic(type_ref);
    let v = pool.get_single_or_panic(value_ref);
    vm.stack.push(StackValue::new(t, v));
    1 + refs(2)
}

macro_rules! handle_i_op {
    ($chunk: ident, $vm: ident, $op: ident) => {{
        let res_ref = $chunk.read_ref(0);
        let op1 = &$vm.stack[$chunk.read_ref(1)];
        let op2 = &$vm.stack[$chunk.read_ref(2)];
        let res = &$vm.stack[res_ref];
        if res.value_type == op1.value_type && op1.value_type == op2.value_type {
            match res.value_type {
                Type::I64 => {
                    let op1 = i64::from_le_bytes(op1.data);
                    let op2 = i64::from_le_bytes(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack[res_ref].data = r.to_le_bytes();
                }
                Type::I32 => {
                    const S: usize = size_of::<i32>();
                    let op1 = i32::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = i32::from_le_bytes(op2.data[..S].try_into().unwrap());
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack[res_ref].data[..S].copy_from_slice(&r.to_le_bytes());
                }
                Type::I16 => {
                    const S: usize = size_of::<i16>();
                    let op1 = i16::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = i16::from_le_bytes(op2.data[..S].try_into().unwrap());
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack[res_ref].data[..S].copy_from_slice(&r.to_le_bytes());
                }
                Type::I8 => {
                    const S: usize = size_of::<i8>();
                    let op1 = i8::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = i8::from_le_bytes(op2.data[..S].try_into().unwrap());
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

macro_rules! handle_u_op {
    ($chunk: ident, $vm: ident, $op: ident) => {{
        let res_ref = $chunk.read_ref(0);
        let op1 = &$vm.stack[$chunk.read_ref(1)];
        let op2 = &$vm.stack[$chunk.read_ref(2)];
        let res = &$vm.stack[res_ref];
        if res.value_type == op1.value_type && op1.value_type == op2.value_type {
            match res.value_type {
                Type::U64 => {
                    let op1 = u64::from_le_bytes(op1.data);
                    let op2 = u64::from_le_bytes(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack[res_ref].data = r.to_le_bytes();
                }
                Type::U32 => {
                    const S: usize = size_of::<u32>();
                    let op1 = u32::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = u32::from_le_bytes(op2.data[..S].try_into().unwrap());
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack[res_ref].data[..S].copy_from_slice(&r.to_le_bytes());
                }
                Type::U16 => {
                    const S: usize = size_of::<u16>();
                    let op1 = u16::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = u16::from_le_bytes(op2.data[..S].try_into().unwrap());
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack[res_ref].data[..S].copy_from_slice(&r.to_le_bytes());
                }
                Type::U8 => {
                    const S: usize = size_of::<u8>();
                    let op1 = u8::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = u8::from_le_bytes(op2.data[..S].try_into().unwrap());
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

macro_rules! handle_f_op {
    ($chunk: ident, $vm: ident, $op: ident) => {{
        let res_ref = $chunk.read_ref(0);
        let op1 = &$vm.stack[$chunk.read_ref(1)];
        let op2 = &$vm.stack[$chunk.read_ref(2)];
        let res = &$vm.stack[res_ref];
        if res.value_type == op1.value_type && op1.value_type == op2.value_type {
            match res.value_type {
                Type::F64 => {
                    let op1 = f64::from_le_bytes(op1.data);
                    let op2 = f64::from_le_bytes(op2.data);
                    let r = op1.$op(op2);
                    $vm.stack[res_ref].data = r.to_le_bytes();
                }
                Type::F32 => {
                    const S: usize = size_of::<f32>();
                    let op1 = f32::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = f32::from_le_bytes(op2.data[..S].try_into().unwrap());
                    let r = op1.$op(op2);
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

pub(super) fn handle_i_add(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_i_op!(chunk, vm, checked_add)
}

pub(super) fn handle_i_sub(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_i_op!(chunk, vm, checked_sub)
}

pub(super) fn handle_i_mul(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_i_op!(chunk, vm, checked_mul)
}

pub(super) fn handle_i_div(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_i_op!(chunk, vm, checked_div)
}

pub(super) fn handle_i_rem(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_i_op!(chunk, vm, checked_rem)
}

pub(super) fn handle_u_add(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_u_op!(chunk, vm, checked_add)
}

pub(super) fn handle_u_sub(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_u_op!(chunk, vm, checked_sub)
}

pub(super) fn handle_u_mul(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_u_op!(chunk, vm, checked_mul)
}

pub(super) fn handle_u_div(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_u_op!(chunk, vm, checked_div)
}

pub(super) fn handle_u_rem(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_u_op!(chunk, vm, checked_rem)
}

pub(super) fn handle_f_add(chunk: &Chunk, vm: &mut Vm) -> usize {
    handle_f_op!(chunk, vm, add)
}

pub(super) fn handle_wide(chunk: &Chunk, vm: &mut Vm) -> usize {
    noop(chunk, vm)
}
const fn refs(n_refs: usize) -> usize {
    n_refs * size_of::<usize>()
}

pub(crate) fn noop(chunk: &Chunk, vm: &mut Vm) -> usize {
    panic!("a bad opcode detected <{}>", chunk.opcode_value());
}
