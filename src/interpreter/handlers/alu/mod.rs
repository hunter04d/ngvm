use crate::interpreter::{three_stack_metadata, two_stack_metadata};
use crate::refs::{TwoRefs, ThreeRefs};
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::{HasVmType, Type};
use crate::refs;
use crate::Vm;
use crate::code::Chunk;

pub mod f_ops;
pub mod i_ops;
pub mod u_ops;

fn process_bi_op<T, R, Func>(vm: &mut Vm, refs: &ThreeRefs, func: Func) -> Option<()>
where
    T: FromSingle<StackData>,
    R: IntoStackData + HasVmType,
    Func: Fn(T, T) -> Option<R>,
{
    let meta = three_stack_metadata(vm, refs)?;
    let op1 = T::from_single(*vm.stack_data(meta.op1.index)?);
    let op2 = T::from_single(*vm.stack_data(meta.op2.index)?);
    let r = func(op1, op2)?;
    let res_index = meta.result.index;
    if meta.result.value_type == R::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return None;
    }
    Some(())
}

fn process_u_op<T, R, Func>(vm: &mut Vm, TwoRefs { result, op }: &TwoRefs, func: Func) -> Option<()>
where
    T: FromSingle<StackData>,
    R: IntoStackData + HasVmType,
    Func: Fn(T) -> Option<R>,
{
    let op = vm.stack_metadata(*op)?;
    let result = vm.stack_metadata(*result)?;
    let op = T::from_single(*vm.stack_data(op.index)?);
    let r = func(op)?;
    let res_index = result.index;
    if result.value_type == R::get_type() {
        *vm.stack_data_mut(res_index)? = r.into_stack_data();
    } else {
        return None;
    }
    Some(())
}


pub(in crate::interpreter) fn handle_b_not(chunk: &Chunk, vm: &mut Vm) -> usize {
    let refs = chunk.read_two().expect("Ok");
    let meta = two_stack_metadata(vm, &refs).unwrap();
    if  meta.result.value_type != Type::Bool {  panic!("Types mismatch")}
    if !meta.op.value_type.is_primitive() { panic!("Invalid operation")}

    let data = *vm.stack_data(meta.op.index).unwrap();
    let res_index = meta.result.index;
    *vm.stack_data_mut(res_index).unwrap() =  data.iter().any(|&v|v != 0u8).into_stack_data();
    1 + refs::refs(2)
}
