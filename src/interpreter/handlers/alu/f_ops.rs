use crate::code::refs::refs_size;
use crate::code::{Chunk, RefSource};
use crate::error::VmError;
use crate::interpreter::handlers::alu::process_u_op;
use crate::operations::markers::*;
use crate::operations::{BiOp, BiOpMarker, UOp, UOpMarker};
use crate::stack::data::IntoStackData;
use crate::types::checker::{HasTypeCheckerCtx, Taggable, TypeCheckerCtx};
use crate::types::{HasPrimitiveType, PrimitiveType, VmType};
use crate::vm::{Vm, VmRefSource};

use super::{process_bi_op, AluExtensions};

fn handle_bi_float_op<M: BiOpMarker>(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError>
where
    f64: BiOp<M>,
    f32: BiOp<M>,
    <f64 as BiOp<M>>::Output: HasPrimitiveType + IntoStackData,
    <f32 as BiOp<M>>::Output: HasPrimitiveType + IntoStackData,
{
    let rf = &chunk.read_three_vm()?;

    let meta = vm.three_stack_metadata(rf)?;
    let mut type_checker_ctx = TypeCheckerCtx::new();

    let t = meta
        .check(&mut type_checker_ctx)
        .all_primitives()
        .all_same()
        .get_vm()?;
    match t {
        PrimitiveType::F64 => process_bi_op::<M, f64>(vm, rf)?,
        PrimitiveType::F32 => process_bi_op::<M, f32>(vm, rf)?,
        _ => return Err(VmError::InvalidTypeForOperation(VmType::from(t).no_tag())),
    }
    Ok(1 + refs_size(3))
}

fn handle_u_float_op<M: UOpMarker>(chunk: &Chunk, vm: &mut Vm) -> Result<(), VmError>
where
    f64: UOp<M>,
    f32: UOp<M>,
    <f64 as UOp<M>>::Output: HasPrimitiveType + IntoStackData,
    <f32 as UOp<M>>::Output: HasPrimitiveType + IntoStackData,
{
    let rf = &chunk.read_two().ok_or(VmError::InvalidBytecode)?;

    let meta = vm.two_stack_metadata(rf)?;
    let t = meta
        .check(&mut TypeCheckerCtx::new())
        .all_primitives()
        .all_same()
        .get_vm()?;
    match t {
        PrimitiveType::F64 => process_u_op::<M, f64>(vm, rf),
        PrimitiveType::F32 => process_u_op::<M, f64>(vm, rf),
        _ => Err(VmError::InvalidTypeForOperation(
            VmType::Primitive(t).no_tag(),
        )),
    }
}

macro_rules! handle_f_ops {
   ($($fn_name: ident => $method_marker: ty),*) => {
        $(
        pub(in crate::interpreter) fn $fn_name(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError>{
            handle_bi_float_op::<$method_marker>(chunk, vm)
        })*

    };
}

handle_f_ops! {
    handle_f_add => Add,
    handle_f_sub => Sub,
    handle_f_mul => Mul,
    handle_f_div => Div,
    handle_f_rem => Rem
}

pub(in crate::interpreter) fn handle_f_neg(chunk: &Chunk, vm: &mut Vm) -> Result<usize, VmError> {
    handle_u_float_op::<Neg>(chunk, vm)?;
    Ok(1 + refs_size(2))
}
