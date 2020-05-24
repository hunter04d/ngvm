use crate::code::Chunk;
use crate::error::VmError;
use crate::operations::markers::{And, Not, Or, Xor};
use crate::operations::{BiOp, BiOpMarker};
use crate::refs::refs_size;
use crate::types::checker::{HasTypeCheckerCtx, Taggable, TypeCheckerCtx};
use crate::types::{PrimitiveType, VmType};
use crate::vm::VmRefSource;
use crate::Vm;

use super::{process_bi_op, process_u_op, AluExtensions};

fn handle_l_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError>
where
    u64: BiOp<M, Output = u64>,
    u32: BiOp<M, Output = u32>,
    u16: BiOp<M, Output = u16>,
    u8: BiOp<M, Output = u8>,
    i64: BiOp<M, Output = i64>,
    i32: BiOp<M, Output = i32>,
    i16: BiOp<M, Output = i16>,
    i8: BiOp<M, Output = i8>,
    bool: BiOp<M, Output = bool>,
{
    let rf = &chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(rf)?;
    let mut type_checker = TypeCheckerCtx::new();
    let t = meta
        .check(&mut type_checker)
        .all_primitives()
        .all_same()
        .get_vm()?;

    match t {
        PrimitiveType::U64 => process_bi_op::<M, u64>(vm, rf),
        PrimitiveType::U32 => process_bi_op::<M, u32>(vm, rf),
        PrimitiveType::U16 => process_bi_op::<M, u16>(vm, rf),
        PrimitiveType::U8 => process_bi_op::<M, u8>(vm, rf),
        PrimitiveType::I64 => process_bi_op::<M, i64>(vm, rf),
        PrimitiveType::I32 => process_bi_op::<M, i32>(vm, rf),
        PrimitiveType::I16 => process_bi_op::<M, i16>(vm, rf),
        PrimitiveType::I8 => process_bi_op::<M, i8>(vm, rf),
        PrimitiveType::Bool => process_bi_op::<M, bool>(vm, rf),
        _ => Err(VmError::InvalidTypeForOperation(VmType::from(t).no_tag())),
    }?;
    Ok(1 + refs_size(3))
}

macro_rules! handle_l_ops {
    ($($fn_name: ident => $marker: ty),* $(,)?) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
            handle_l_op::<$marker>(chunk, vm)
        }
        )*
    };
}

handle_l_ops! {
    handle_l_and => And,
    handle_l_or => Or,
    handle_l_xor => Xor,
}

pub(in crate::interpreter) fn handle_l_not(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    let rf = &chunk.read_two_vm()?;

    let meta = vm.two_stack_metadata(rf)?;

    let t_ctx = &mut TypeCheckerCtx::new();
    let t = meta
        .check(t_ctx)
        .all_primitives()
        .all_same()
        .and()
        .either()
        .integer()
        .or()
        .bool()
        .fmt(|_, t| format!("<{}>{:?} is neither integer or bool", t.tag, t.vm_type))
        .get_vm()?;

    match t {
        PrimitiveType::U64 => process_u_op::<Not, u64>(vm, rf),
        PrimitiveType::U32 => process_u_op::<Not, u32>(vm, rf),
        PrimitiveType::U16 => process_u_op::<Not, u16>(vm, rf),
        PrimitiveType::U8 => process_u_op::<Not, u8>(vm, rf),
        PrimitiveType::I64 => process_u_op::<Not, i64>(vm, rf),
        PrimitiveType::I32 => process_u_op::<Not, i32>(vm, rf),
        PrimitiveType::I16 => process_u_op::<Not, i16>(vm, rf),
        PrimitiveType::I8 => process_u_op::<Not, i8>(vm, rf),
        PrimitiveType::Bool => process_u_op::<Not, bool>(vm, rf),
        _ => return Err(VmError::InvalidTypeForOperation(VmType::from(t).no_tag())),
    }?;

    Ok(1 + refs_size(3))
}
