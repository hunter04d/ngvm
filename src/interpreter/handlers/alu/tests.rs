//! Tests of the arithmetic ops: one opcode per operation, dispatched by the operand type

use crate::code::refs::{PoolRef, StackRef, p, three, two};
use crate::error::VmError;
use crate::model::Opcode::{self, *};
use crate::stack::data::{IntoPrimitive, StackData};
use crate::types::PrimitiveType::{self, *};
use crate::{Code, Constant, ConstantPool, Vm};

/// | index | constant   |
/// |-------|------------|
/// | 0     | type u8    |
/// | 1     | type i32   |
/// | 2     | type u64   |
/// | 3     | type f64   |
/// | 4     | type f32   |
/// | 5     | 255u8      |
/// | 6     | 1u8        |
/// | 7     | -7i32      |
/// | 8     | 2i32       |
/// | 9     | 0i32       |
/// | 10    | 1.5f64     |
/// | 11    | 2.25f64    |
/// | 12    | 0f64       |
/// | 13    | 1.5f32     |
/// | 14    | 3u64       |
fn pool() -> ConstantPool {
    ConstantPool::new(vec![
        U8.into(),
        I32.into(),
        U64.into(),
        F64.into(),
        F32.into(),
        255u8.into(),
        1u8.into(),
        Constant::from(-7i32),
        2i32.into(),
        0i32.into(),
        1.5f64.into(),
        2.25f64.into(),
        0f64.into(),
        1.5f32.into(),
        3u64.into(),
    ])
}

fn type_location(t: PrimitiveType) -> PoolRef {
    match t {
        U8 => p(0),
        I32 => p(1),
        U64 => p(2),
        F64 => p(3),
        F32 => p(4),
        _ => unreachable!("not in the pool"),
    }
}

fn ld(t: PrimitiveType, value: usize) -> Opcode {
    LDType {
        type_location: type_location(t),
        value_location: p(value),
    }
}

fn ld0(t: PrimitiveType) -> Opcode {
    LdTyped0 {
        type_location: type_location(t),
    }
}

/// Loads `op1` and `op2` at @0 and @1, a result slot of type `t` at @2, then runs `op`
fn bi(t: PrimitiveType, op1: usize, op2: usize, op: Opcode) -> Vec<Opcode> {
    vec![ld(t, op1), ld(t, op2), ld0(t), op]
}

/// Loads `op` at @0 and a result slot of type `t` at @1, then runs `neg`
fn neg(t: PrimitiveType, op: usize) -> Vec<Opcode> {
    vec![ld(t, op), ld0(t), Neg(two(1, 0))]
}

fn run(code: Vec<Opcode>) -> (Vm, Result<(), VmError>) {
    let mut vm = Vm::headless(pool());
    let code = Code::from_model(&code).expect("valid model");
    let result = code.interpret(&mut vm).map_err(|e| e.error);
    (vm, result)
}

fn run_at<T>(code: Vec<Opcode>, index: usize) -> T
where
    StackData: IntoPrimitive<T>,
{
    let (vm, result) = run(code);
    if let Err(e) = result {
        panic!("program failed: {:?}", e);
    }
    vm.single_stack_data(StackRef(index))
        .unwrap()
        .into_primitive()
}

fn run_err(code: Vec<Opcode>) -> VmError {
    run(code).1.expect_err("program must fail")
}

fn assert_type_error(e: VmError) {
    assert!(
        matches!(e, VmError::TypeError(_)),
        "expected a type error, got {:?}",
        e
    );
}

#[test]
fn test_add_dispatches_by_operand_type() {
    assert_eq!(run_at::<u8>(bi(U8, 6, 6, Add(three(2, 0, 1))), 2), 2);
    assert_eq!(run_at::<i32>(bi(I32, 7, 8, Add(three(2, 0, 1))), 2), -5);
    assert_eq!(run_at::<f64>(bi(F64, 10, 11, Add(three(2, 0, 1))), 2), 3.75);
}

#[test]
fn test_ops_compute_the_right_values() {
    assert_eq!(run_at::<i32>(bi(I32, 7, 8, Sub(three(2, 0, 1))), 2), -9);
    assert_eq!(run_at::<i32>(bi(I32, 7, 8, Mul(three(2, 0, 1))), 2), -14);
    assert_eq!(run_at::<i32>(bi(I32, 7, 8, Div(three(2, 0, 1))), 2), -3);
    assert_eq!(run_at::<i32>(bi(I32, 7, 8, Rem(three(2, 0, 1))), 2), -1);
    assert_eq!(run_at::<f64>(bi(F64, 11, 10, Rem(three(2, 0, 1))), 2), 0.75);
}

#[test]
fn test_integer_ops_are_checked() {
    assert!(matches!(
        run_err(bi(U8, 5, 6, Add(three(2, 0, 1)))),
        VmError::BiOpError
    ));
    assert!(matches!(
        run_err(bi(I32, 7, 9, Div(three(2, 0, 1)))),
        VmError::BiOpError
    ));
}

#[test]
fn test_float_ops_follow_ieee() {
    let r = run_at::<f64>(bi(F64, 10, 12, Div(three(2, 0, 1))), 2);
    assert_eq!(r, f64::INFINITY);
}

#[test]
fn test_neg_signed_and_float() {
    assert_eq!(run_at::<i32>(neg(I32, 7), 1), 7);
    assert_eq!(run_at::<f64>(neg(F64, 10), 1), -1.5);
    assert_eq!(run_at::<f32>(neg(F32, 13), 1), -1.5);
}

#[test]
fn test_neg_rejects_unsigned() {
    assert_type_error(run_err(neg(U8, 6)));
}

#[test]
fn test_arith_rejects_non_numbers() {
    let code = vec![LdTrue, LdTrue, LdFalse, Add(three(2, 0, 1))];
    assert_type_error(run_err(code));
}

#[test]
fn test_arith_rejects_mixed_types() {
    let code = vec![ld(U64, 14), ld(I32, 8), ld(U64, 14), Add(three(2, 0, 1))];
    assert_type_error(run_err(code));
}

