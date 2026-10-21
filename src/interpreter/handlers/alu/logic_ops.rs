use crate::Vm;
use crate::code::refs::ThreeStackRefs;
use crate::code::{Chunk, refs::refs_size};
use crate::error::VmError;
use crate::types::PrimitiveType;
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx};
use crate::vm::VmRefSource;

use super::{AluExtensions, process_bi_op, process_u_op};

/// Reads the operands of a binary logic op and checks they are all the same integer or bool type
fn bi_logic_operands(chunk: &Chunk, vm: &Vm) -> Result<(ThreeStackRefs, PrimitiveType), VmError> {
    let rf = chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(&rf)?;
    let mut type_checker = TypeCheckerCtx::new();
    let t = meta
        .check(&mut type_checker)
        .all_primitives()
        .all_same()
        .either()
        .integer()
        .or()
        .bool()
        .and()
        .fmt(|_, t| format!("<{}>{:?} is neither integer or bool", t.tag, t.vm_type))
        .get_vm()?
        .op1;
    Ok((rf, t))
}

macro_rules! handle_l_ops {
    ($($fn_name: ident => $op: tt);* $(;)?) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            let (rf, t) = &bi_logic_operands(chunk, vm)?;
            match t {
                PrimitiveType::U64 => process_bi_op(vm, rf, |a: u64, b: u64| a $op b),
                PrimitiveType::U32 => process_bi_op(vm, rf, |a: u32, b: u32| a $op b),
                PrimitiveType::U16 => process_bi_op(vm, rf, |a: u16, b: u16| a $op b),
                PrimitiveType::U8 => process_bi_op(vm, rf, |a: u8, b: u8| a $op b),
                PrimitiveType::I64 => process_bi_op(vm, rf, |a: i64, b: i64| a $op b),
                PrimitiveType::I32 => process_bi_op(vm, rf, |a: i32, b: i32| a $op b),
                PrimitiveType::I16 => process_bi_op(vm, rf, |a: i16, b: i16| a $op b),
                PrimitiveType::I8 => process_bi_op(vm, rf, |a: i8, b: i8| a $op b),
                PrimitiveType::Bool => process_bi_op(vm, rf, |a: bool, b: bool| a $op b),
                _ => unreachable!("{t:?} passed the integer or bool check"),
            }?;
            Ok(1 + refs_size(3))
        }
        )*
    };
}

handle_l_ops! {
    handle_l_and => &;
    handle_l_or => |;
    handle_l_xor => ^;
}

pub(in crate::interpreter) fn handle_l_not(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let rf = &chunk.read_two_vm()?;

    let meta = vm.two_stack_metadata(rf)?;

    let t_ctx = &mut TypeCheckerCtx::new();
    let t = meta
        .check(t_ctx)
        .all_primitives()
        .all_same()
        .either()
        .integer()
        .or()
        .bool()
        .and()
        .fmt(|_, t| format!("<{}>{:?} is neither integer or bool", t.tag, t.vm_type))
        .get_vm()?
        .op;

    match t {
        PrimitiveType::U64 => process_u_op(vm, rf, |a: u64| !a),
        PrimitiveType::U32 => process_u_op(vm, rf, |a: u32| !a),
        PrimitiveType::U16 => process_u_op(vm, rf, |a: u16| !a),
        PrimitiveType::U8 => process_u_op(vm, rf, |a: u8| !a),
        PrimitiveType::I64 => process_u_op(vm, rf, |a: i64| !a),
        PrimitiveType::I32 => process_u_op(vm, rf, |a: i32| !a),
        PrimitiveType::I16 => process_u_op(vm, rf, |a: i16| !a),
        PrimitiveType::I8 => process_u_op(vm, rf, |a: i8| !a),
        PrimitiveType::Bool => process_u_op(vm, rf, |a: bool| !a),
        _ => unreachable!("{t:?} passed the integer or bool check"),
    }?;

    Ok(1 + refs_size(2))
}
