use std::cmp::Ordering;

use crate::Vm;
use crate::code::Chunk;
use crate::code::refs::{ThreeStackRefs, refs_size};
use crate::error::VmError;
use crate::interpreter::handlers::alu::AluExtensions;
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::PrimitiveType;
use crate::types::checker::{HasTypeCheckerCtx, TypeCheckerCtx};
use crate::vm::VmRefSource;

fn handle_cmp_op(
    chunk: &Chunk,
    vm: &mut Vm,
    to_bool: impl Fn(Option<Ordering>) -> bool,
) -> Result<usize, VmError> {
    let rf = &chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(rf)?;
    let mut type_checker = TypeCheckerCtx::new();
    let t = meta
        .check(&mut type_checker)
        .all_primitives()
        .result()
        .bool()
        .and()
        .same_operands()
        .number()
        .get_vm()?;

    match t {
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
        _ => unreachable!("{t:?} passed the number check"),
    }?;
    Ok(1 + refs_size(3))
}

// `None` means the operands are unordered (a NaN is involved): every comparison is false but `ne`,
// as in IEEE 754

fn eq(o: Option<Ordering>) -> bool {
    matches!(o, Some(Ordering::Equal))
}

fn ne(o: Option<Ordering>) -> bool {
    !eq(o)
}

fn le(o: Option<Ordering>) -> bool {
    matches!(o, Some(Ordering::Less | Ordering::Equal))
}

fn ge(o: Option<Ordering>) -> bool {
    matches!(o, Some(Ordering::Greater | Ordering::Equal))
}

fn lt(o: Option<Ordering>) -> bool {
    matches!(o, Some(Ordering::Less))
}

fn gt(o: Option<Ordering>) -> bool {
    matches!(o, Some(Ordering::Greater))
}

fn process_cmp_op<T, F: Fn(Option<Ordering>) -> bool>(
    vm: &mut Vm,
    refs: &ThreeStackRefs,
    to_bool: F,
) -> Result<(), VmError>
where
    T: FromSingle<StackData> + PartialOrd,
{
    let op1 = T::from_single(*vm.single_stack_data(refs.op1)?);
    let op2 = T::from_single(*vm.single_stack_data(refs.op2)?);
    let r = op1.partial_cmp(&op2);
    vm.set_single_stack_data(refs.result, to_bool(r).into_stack_data())?;
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
