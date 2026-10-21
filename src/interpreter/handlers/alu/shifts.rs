use crate::code::{Chunk, refs::refs_size};
use crate::error::VmError;
use crate::interpreter::handlers::alu::{AluExtensions, process_fallible_bi_op};
use crate::operations::markers::*;
use crate::operations::{BiOp, BiOpMarker};
use crate::types::PrimitiveType;
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx, tags};
use crate::vm::{Vm, VmRefSource};

fn handle_shift_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError>
where
    u64: BiOp<M, u32>,
    u32: BiOp<M, u32>,
    u16: BiOp<M, u32>,
    u8: BiOp<M, u32>,
    i64: BiOp<M, u32>,
    i32: BiOp<M, u32>,
    i16: BiOp<M, u32>,
    i8: BiOp<M, u32>,
    <u64 as BiOp<M, u32>>::Output: Into<Option<u64>>,
    <u32 as BiOp<M, u32>>::Output: Into<Option<u32>>,
    <u16 as BiOp<M, u32>>::Output: Into<Option<u16>>,
    <u8 as BiOp<M, u32>>::Output: Into<Option<u8>>,
    <i64 as BiOp<M, u32>>::Output: Into<Option<i64>>,
    <i32 as BiOp<M, u32>>::Output: Into<Option<i32>>,
    <i16 as BiOp<M, u32>>::Output: Into<Option<i16>>,
    <i8 as BiOp<M, u32>>::Output: Into<Option<i8>>,
{
    let rf = &chunk.read_three_vm()?;
    // the shift amount is read as u32, which is correct for the narrower types only because the
    // stack stores values zero-extended

    let meta = vm.three_stack_metadata(rf)?;
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

    match types.op1 {
        PrimitiveType::U64 => process_fallible_bi_op::<M, u64, u32>(vm, rf),
        PrimitiveType::U32 => process_fallible_bi_op::<M, u32, u32>(vm, rf),
        PrimitiveType::U16 => process_fallible_bi_op::<M, u16, u32>(vm, rf),
        PrimitiveType::U8 => process_fallible_bi_op::<M, u8, u32>(vm, rf),
        PrimitiveType::I64 => process_fallible_bi_op::<M, i64, u32>(vm, rf),
        PrimitiveType::I32 => process_fallible_bi_op::<M, i32, u32>(vm, rf),
        PrimitiveType::I16 => process_fallible_bi_op::<M, i16, u32>(vm, rf),
        PrimitiveType::I8 => process_fallible_bi_op::<M, i8, u32>(vm, rf),
        t => unreachable!("{t:?} passed the integer check"),
    }?;
    Ok(1 + refs_size(3))
}

macro_rules! handle_shifts {
    ($($fn_name: ident => $marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            handle_shift_op::<$marker>(chunk, vm)
        }
        )*
    };
}

handle_shifts! {
    handle_shl => CheckedShl,
    handle_shr => CheckedShr,
    handle_rotl => CheckedRotL,
    handle_rotr => CheckedRotR
}
