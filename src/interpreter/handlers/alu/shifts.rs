use crate::code::refs::ThreeStackRefs;
use crate::code::{Chunk, refs::refs_size};
use crate::error::VmError;
use crate::interpreter::handlers::alu::{AluExtensions, process_bi_op, process_fallible_bi_op};
use crate::types::PrimitiveType;
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx, tags};
use crate::vm::{Vm, VmRefSource};

/// Reads the operands of a shift and checks the shifted value is an integer of the result type
fn shift_operands(chunk: &Chunk, vm: &Vm) -> Result<(ThreeStackRefs, PrimitiveType), VmError> {
    let rf = chunk.read_three_vm()?;
    // the shift amount is read as u32, which is correct for the narrower types only because the
    // stack stores values zero-extended

    let meta = vm.three_stack_metadata(&rf)?;
    let mut type_checker = TypeCheckerCtx::new();
    let types = meta
        .check(&mut type_checker)
        .all_primitives()
        .op2()
        .one_of(&[PrimitiveType::U32, PrimitiveType::U16, PrimitiveType::U8])
        .and()
        .same(&[tags::RESULT, tags::OP1])
        .integer()
        .and()
        .get_vm()?;
    Ok((rf, types.op1))
}

macro_rules! handle_shifts {
    ($($fn_name: ident => $process: ident, $method: ident);* $(;)?) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            let (rf, t) = &shift_operands(chunk, vm)?;
            match t {
                PrimitiveType::U64 => $process(vm, rf, u64::$method),
                PrimitiveType::U32 => $process(vm, rf, u32::$method),
                PrimitiveType::U16 => $process(vm, rf, u16::$method),
                PrimitiveType::U8 => $process(vm, rf, u8::$method),
                PrimitiveType::I64 => $process(vm, rf, i64::$method),
                PrimitiveType::I32 => $process(vm, rf, i32::$method),
                PrimitiveType::I16 => $process(vm, rf, i16::$method),
                PrimitiveType::I8 => $process(vm, rf, i8::$method),
                t => unreachable!("{t:?} passed the integer check"),
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
