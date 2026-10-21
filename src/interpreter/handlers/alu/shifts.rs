use crate::code::refs::ThreeStackRefs;
use crate::code::{Chunk, refs::refs_size};
use crate::error::VmError;
use crate::interpreter::handlers::alu::{AluExtensions, process_bi_op, process_fallible_bi_op};
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx};
use crate::types::{IntegerType, PrimitiveType};
use crate::vm::{Vm, VmRefSource};

/// Reads the operands of a shift and checks the shifted value is an integer of the result type
fn shift_operands(chunk: &Chunk, vm: &Vm) -> Result<(ThreeStackRefs, IntegerType), VmError> {
    let rf = chunk.read_three_vm()?;
    // the shift amount is read as u32, which is correct for the narrower types only because the
    // stack stores values zero-extended

    let meta = vm.three_stack_metadata(&rf)?;
    let mut type_checker = TypeCheckerCtx::new();
    let t = meta
        .check(&mut type_checker)
        .same_result_and_op1()
        .result(|t| t.integer())
        .op(|t| t.one_of(&[PrimitiveType::U32, PrimitiveType::U16, PrimitiveType::U8]))
        .get_vm()?
        .result;
    Ok((rf, t))
}

macro_rules! handle_shifts {
    ($($fn_name: ident => $process: ident, $method: ident);* $(;)?) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            let (rf, t) = &shift_operands(chunk, vm)?;
            match t {
                IntegerType::U64 => $process(vm, rf, u64::$method),
                IntegerType::U32 => $process(vm, rf, u32::$method),
                IntegerType::U16 => $process(vm, rf, u16::$method),
                IntegerType::U8 => $process(vm, rf, u8::$method),
                IntegerType::I64 => $process(vm, rf, i64::$method),
                IntegerType::I32 => $process(vm, rf, i32::$method),
                IntegerType::I16 => $process(vm, rf, i16::$method),
                IntegerType::I8 => $process(vm, rf, i8::$method),
            }?;
            Ok(1 + refs_size(3))
        }
        )*
    };
}

handle_shifts! {
    handle_shl => process_fallible_bi_op, checked_shl;
    handle_shr => process_fallible_bi_op, checked_shr;
    handle_rotl => process_bi_op, rotate_left;
    handle_rotr => process_bi_op, rotate_right;
}
