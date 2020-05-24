use std::cmp::Ordering;

use crate::code::Chunk;
use crate::error::VmError;
use crate::interpreter::handlers::alu::AluExtensions;
use crate::refs::{refs_size, ThreeStackRefs};
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::checker::{HasTypeCheckerCtx, Taggable, TypeCheckerCtx};
use crate::types::{PrimitiveType, VmType};
use crate::vm::VmRefSource;
use crate::Vm;

fn handle_cmp_op(
    chunk: &Chunk,
    vm: &mut Vm,
    to_bool: impl Fn(Ordering) -> bool,
) -> Result<usize, VmError> {
    let rf = &chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(rf)?;
    let mut type_checker = TypeCheckerCtx::new();
    let types = meta
        .check(&mut type_checker)
        .all_primitives()
        .result()
        .bool()
        .operands()
        .same()
        .get_vm()?;

    match types.op {
        PrimitiveType::U64 => process_cmp_op::<u64, _>(vm, rf, to_bool),
        PrimitiveType::U32 => process_cmp_op::<u32, _>(vm, rf, to_bool),
        PrimitiveType::U16 => process_cmp_op::<u16, _>(vm, rf, to_bool),
        PrimitiveType::U8 => process_cmp_op::<u8, _>(vm, rf, to_bool),
        PrimitiveType::I64 => process_cmp_op::<i64, _>(vm, rf, to_bool),
        PrimitiveType::I32 => process_cmp_op::<i32, _>(vm, rf, to_bool),
        PrimitiveType::I16 => process_cmp_op::<i16, _>(vm, rf, to_bool),
        PrimitiveType::I8 => process_cmp_op::<i8, _>(vm, rf, to_bool),
        PrimitiveType::F64 => process_cmp_op::<f64, _>(vm, rf, to_bool),
        PrimitiveType::F32 => process_cmp_op::<f32, _>(vm, rf, to_bool),
        _ => Err(VmError::InvalidTypeForOperation(
            VmType::from(types.op).no_tag(),
        )),
    }?;
    Ok(1 + refs_size(3))
}

fn eq(o: Ordering) -> bool {
    matches!(o, Ordering::Equal)
}

fn ne(o: Ordering) -> bool {
    !eq(o)
}

fn le(o: Ordering) -> bool {
    matches!(o, Ordering::Less | Ordering::Equal)
}

fn ge(o: Ordering) -> bool {
    matches!(o, Ordering::Greater | Ordering::Equal)
}

fn lt(o: Ordering) -> bool {
    matches!(o, Ordering::Less)
}

fn gt(o: Ordering) -> bool {
    matches!(o, Ordering::Greater)
}

fn process_cmp_op<T, F: Fn(Ordering) -> bool>(
    vm: &mut Vm,
    refs: &ThreeStackRefs,
    to_bool: F,
) -> Result<(), VmError>
where
    T: FromSingle<StackData> + PartialOrd,
{
    let meta = vm.three_stack_metadata(refs)?;
    let op1 = T::from_single(*vm.stack_data(meta.op1.index)?);
    let op2 = T::from_single(*vm.stack_data(meta.op2.index)?);
    let r = op1.partial_cmp(&op2).ok_or(VmError::BiOpError)?;
    let res_index = meta.result.index;
    *vm.stack_data_mut(res_index)? = to_bool(r).into_stack_data();
    Ok(())
}

pub(in crate::interpreter) fn handle_eq(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_cmp_op(chunk, vm, eq)
}

pub(in crate::interpreter) fn handle_ne(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_cmp_op(chunk, vm, ne)
}

pub(in crate::interpreter) fn handle_lt(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_cmp_op(chunk, vm, lt)
}

pub(in crate::interpreter) fn handle_le(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_cmp_op(chunk, vm, le)
}

pub(in crate::interpreter) fn handle_gt(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_cmp_op(chunk, vm, gt)
}

pub(in crate::interpreter) fn handle_ge(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_cmp_op(chunk, vm, ge)
}
