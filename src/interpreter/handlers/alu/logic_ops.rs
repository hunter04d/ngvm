use crate::Vm;
use crate::code::refs::ThreeStackRefs;
use crate::code::{Chunk, refs::refs_size};
use crate::error::VmError;
use crate::types::IntegerOrBoolType;
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx};
use crate::vm::VmRefSource;

use super::{AluExtensions, process_bi_op, process_u_op};

/// Reads the operands of a binary logic op and checks they are all the same integer or bool type
fn bi_logic_operands(
    chunk: &Chunk,
    vm: &Vm,
) -> Result<(ThreeStackRefs, IntegerOrBoolType), VmError> {
    let rf = chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(&rf)?;
    let mut type_checker = TypeCheckerCtx::new();
    let t = meta
        .check(&mut type_checker)
        .all_same()
        .is::<IntegerOrBoolType>()
        .get_vm()?;
    Ok((rf, t))
}

macro_rules! handle_l_ops {
    ($($fn_name: ident => $op: tt);* $(;)?) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            let (rf, t) = &bi_logic_operands(chunk, vm)?;
            match t {
                IntegerOrBoolType::U64 => process_bi_op(vm, rf, |a: u64, b: u64| a $op b),
                IntegerOrBoolType::U32 => process_bi_op(vm, rf, |a: u32, b: u32| a $op b),
                IntegerOrBoolType::U16 => process_bi_op(vm, rf, |a: u16, b: u16| a $op b),
                IntegerOrBoolType::U8 => process_bi_op(vm, rf, |a: u8, b: u8| a $op b),
                IntegerOrBoolType::I64 => process_bi_op(vm, rf, |a: i64, b: i64| a $op b),
                IntegerOrBoolType::I32 => process_bi_op(vm, rf, |a: i32, b: i32| a $op b),
                IntegerOrBoolType::I16 => process_bi_op(vm, rf, |a: i16, b: i16| a $op b),
                IntegerOrBoolType::I8 => process_bi_op(vm, rf, |a: i8, b: i8| a $op b),
                IntegerOrBoolType::Bool => process_bi_op(vm, rf, |a: bool, b: bool| a $op b),
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
        .all_same()
        .is::<IntegerOrBoolType>()
        .get_vm()?;

    match t {
        IntegerOrBoolType::U64 => process_u_op(vm, rf, |a: u64| !a),
        IntegerOrBoolType::U32 => process_u_op(vm, rf, |a: u32| !a),
        IntegerOrBoolType::U16 => process_u_op(vm, rf, |a: u16| !a),
        IntegerOrBoolType::U8 => process_u_op(vm, rf, |a: u8| !a),
        IntegerOrBoolType::I64 => process_u_op(vm, rf, |a: i64| !a),
        IntegerOrBoolType::I32 => process_u_op(vm, rf, |a: i32| !a),
        IntegerOrBoolType::I16 => process_u_op(vm, rf, |a: i16| !a),
        IntegerOrBoolType::I8 => process_u_op(vm, rf, |a: i8| !a),
        IntegerOrBoolType::Bool => process_u_op(vm, rf, |a: bool| !a),
    }?;

    Ok(1 + refs_size(2))
}
