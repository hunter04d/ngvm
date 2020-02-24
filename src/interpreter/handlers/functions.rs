use std::convert::TryInto;
use std::mem::size_of;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use crate::code::Chunk;
use crate::interpreter::StackTracer;
use crate::opcodes::refs;
use crate::types::Type;
use crate::{StackValue, Vm};

pub(super) fn handle_u64_ld0(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(StackValue::default_with_type(Type::U64));
    1
}

pub(super) fn handle_i64_ld0(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(StackValue::default_with_type(Type::I64));
    1
}

pub(super) fn handle_ld_unit(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(StackValue::default_with_type(Type::Unit));
    1
}

pub(super) fn handle_ld_typed0(chunk: &Chunk, vm: &mut Vm) -> usize {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref(0);
    let t = pool.get_type_or_panic(type_ref);
    vm.push_single_stack_value(StackValue::new(t, Default::default()));
    1 + refs(1)
}
pub(super) fn handle_ld_type(chunk: &Chunk, vm: &mut Vm) -> usize {
    let pool = vm.current_const_pool();
    let type_ref = chunk.read_ref(0);
    let value_ref = chunk.read_ref(1);
    let t = pool.get_type_or_panic(type_ref);
    let v = pool.get_single_or_panic(value_ref);
    vm.push_single_stack_value(StackValue::new(t, v));
    1 + refs(2)
}

macro_rules! handle_i_op {
    ($chunk: ident, $vm: ident, $op: ident) => {{
        let res_ref = $chunk.read_ref(0);
        let op1 = $vm.stack_value($chunk.read_ref(1));
        let op2 = $vm.stack_value($chunk.read_ref(2));
        let res = $vm.stack_value(res_ref);
        if res.value_type == op1.value_type && op1.value_type == op2.value_type {
            match res.value_type {
                Type::I64 => {
                    let op1 = i64::from_le_bytes(op1.data);
                    let op2 = i64::from_le_bytes(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack_value_mut(res_ref).data = r.to_le_bytes();
                }
                Type::I32 => {
                    const S: usize = size_of::<i32>();
                    let op1 = i32::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = i32::from_le_bytes(op2.data[..S].try_into().unwrap());
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
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
        let op1 = $vm.stack_value($chunk.read_ref(1));
        let op2 = $vm.stack_value($chunk.read_ref(2));
        let res = $vm.stack_value(res_ref);
        if res.value_type == op1.value_type && op1.value_type == op2.value_type {
            match res.value_type {
                Type::U64 => {
                    let op1 = u64::from_le_bytes(op1.data);
                    let op2 = u64::from_le_bytes(op2.data);
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack_value_mut(res_ref).data = r.to_le_bytes();
                }
                Type::U32 => {
                    const S: usize = size_of::<u32>();
                    let op1 = u32::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = u32::from_le_bytes(op2.data[..S].try_into().unwrap());
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
                }
                Type::U16 => {
                    const S: usize = size_of::<u16>();
                    let op1 = u16::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = u16::from_le_bytes(op2.data[..S].try_into().unwrap());
                    let r = op1.$op(op2).expect("Overflow error");
                    $vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
                }
                Type::U8 => {
                    const S: usize = size_of::<u8>();
                    let op1 = u8::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = u8::from_le_bytes(op2.data[..S].try_into().unwrap());
                    let r = op1.$op(op2).expect("Overflow error");
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
                    $vm.stack_value_mut(res_ref).data = r.to_le_bytes();
                }
                Type::F32 => {
                    const S: usize = size_of::<f32>();
                    let op1 = f32::from_le_bytes(op1.data[..S].try_into().unwrap());
                    let op2 = f32::from_le_bytes(op2.data[..S].try_into().unwrap());
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

macro_rules! handle_u_ops {
   ($($fn_name: ident => $method_name: ident),*) => {
        $(
        pub(in crate::interpreter::handlers) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_u_op!(chunk, vm, $method_name)
        })*

    };
}

macro_rules! handle_i_ops {
   ($($fn_name: ident => $method_name: ident),*) => {
        $(
        pub(in crate::interpreter::handlers) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_i_op!(chunk, vm, $method_name)
        })*

    };
}

macro_rules! handle_f_ops {
   ($($fn_name: ident => $method_name: ident),*) => {
        $(
        pub(in crate::interpreter::handlers) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> usize {
            handle_f_op!(chunk, vm, $method_name)
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

handle_i_ops! {
    handle_i_add => checked_add,
    handle_i_sub => checked_sub,
    handle_i_mul => checked_mul,
    handle_i_div => checked_div,
    handle_i_rem => checked_rem
}

handle_f_ops! {
    handle_f_add => add,
    handle_f_sub => sub,
    handle_f_mul => mul,
    handle_f_div => div,
    handle_f_rem => rem
}

pub(super) fn handle_i_neg(chunk: &Chunk, vm: &mut Vm) -> usize {
    let res_ref = chunk.read_ref(0);
    let op_ref = chunk.read_ref(1);
    let op = vm.stack_value(op_ref);
    let res = vm.stack_value(res_ref);
    if op.value_type == res.value_type {
        match res.value_type {
            Type::I64 => {
                let op: i64 = i64::from_le_bytes(op.data);
                vm.stack_value_mut(res_ref).data = i64::to_le_bytes(op.neg());
            }
            Type::I32 => {
                const S: usize = size_of::<i32>();
                let r = -i32::from_le_bytes(op.data[..S].try_into().unwrap());
                vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
            }
            Type::I16 => {
                const S: usize = size_of::<i16>();
                let r = -i16::from_le_bytes(op.data[..S].try_into().unwrap());
                vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
            }
            Type::I8 => {
                const S: usize = size_of::<i8>();
                let r = -i8::from_le_bytes(op.data[..S].try_into().unwrap());
                vm.stack_value_mut(res_ref).data[..S].copy_from_slice(&r.to_le_bytes());
            }
            _ => panic!("Type mismatch"),
        }
    } else {
        panic!("Type mismatch")
    }

    1 + refs(2)
}

pub(super) fn handle_f_neg(chunk: &Chunk, vm: &mut Vm) -> usize {
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

pub(super) fn handle_ld_true(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(true.into());
    1
}

pub(super) fn handle_ld_false(_: &Chunk, vm: &mut Vm) -> usize {
    vm.push_single_stack_value(false.into());
    1
}

/// For debug only
pub(super) fn handle_trace_stack_value(chunk: &Chunk, vm: &mut Vm) -> usize {
    let stack_ref = chunk.read_ref(0);
    eprintln!(
        "Trace @{}: {:#?}",
        stack_ref,
        StackTracer(&vm.stack[stack_ref..])
    );
    1 + refs(1)
}

pub(super) fn handle_wide(chunk: &Chunk, vm: &mut Vm) -> usize {
    noop(chunk, vm)
}

pub(crate) fn noop(chunk: &Chunk, _vm: &mut Vm) -> usize {
    panic!("a bad opcode detected <{}>", chunk.opcode_value());
}
