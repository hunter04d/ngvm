//! Tests of the memory model: borrows, moves, scopes and derefs

use crate::code::refs::{p, s, three, two, StackRef};
use crate::error::{VmContextError, VmError};
use crate::model::Opcode::{self, *};
use crate::stack::data::{IntoPrimitive, StackData};
use crate::types::PrimitiveType::*;
use crate::vm::lock::{LockError, ValueLock};
use crate::{Code, Constant, ConstantPool, Vm};

const STR: &str = "str";

/// | index | constant        |
/// |-------|-----------------|
/// | 0     | type u64        |
/// | 1     | 0               |
/// | 2     | 1               |
/// | 3     | 3               |
/// | 4     | type SStr       |
/// | 5     | `STR`           |
/// | 6     | type Never      |
/// | 7     | type Char       |
/// | 8     | invalid char    |
fn pool() -> ConstantPool {
    ConstantPool::new(vec![
        U64.into(),
        0u64.into(),
        1u64.into(),
        3u64.into(),
        SStr.into(),
        STR.into(),
        Never.into(),
        Char.into(),
        Constant::from(0xD800u32),
    ])
}

fn ld_u64(value: u64) -> Opcode {
    let value_location = match value {
        0 => p(1),
        1 => p(2),
        3 => p(3),
        _ => unreachable!("not in the pool"),
    };
    LDType {
        type_location: p(0),
        value_location,
    }
}

fn run(code: Vec<Opcode>) -> (Vm, Result<(), VmContextError>) {
    let mut vm = Vm::headless(pool());
    let code = Code::from_model(&code).expect("valid model");
    let result = code.interpret(&mut vm);
    (vm, result)
}

/// Runs a program that must succeed, checks that no borrow is left behind
fn run_ok(code: Vec<Opcode>) -> Vm {
    let (vm, result) = run(code);
    if let Err(e) = result {
        panic!("program failed: {:?}", e);
    }
    assert!(vm.derefs.is_empty());
    assert!(vm.transient_refs.is_empty());
    for meta in &vm.stack_metadata {
        assert_eq!(meta.lock, ValueLock::None, "{:?}", meta);
    }
    vm
}

fn run_err(code: Vec<Opcode>) -> VmError {
    run(code).1.expect_err("program must fail").error
}

fn u64_at(vm: &Vm, index: usize) -> u64 {
    vm.single_stack_data(StackRef(index)).unwrap().into_primitive()
}

fn data_at(vm: &Vm, index: usize) -> Vec<u64> {
    vm.stack_data(StackRef(index))
        .unwrap()
        .iter()
        .map(|d: &StackData| d.into_primitive())
        .collect()
}

fn assert_lock_error(e: VmError, expected: LockError) {
    match e {
        VmError::LockError(e, _) => assert_eq!(format!("{e:?}"), format!("{expected:?}")),
        e => panic!("expected {:?}, got {:?}", expected, e),
    }
}

#[test]
fn test_deref_writes_through_mut_ref() {
    let vm = run_ok(vec![
        Ld0U64,
        Scope(vec![
            TakeMut(s(0)),
            ld_u64(1),
            StartDeref(s(1)),
            UAdd(three(3, 3, 2)),
            EndDeref,
        ]),
    ]);
    assert_eq!(u64_at(&vm, 0), 1);
}

#[test]
fn test_nested_refs() {
    run_ok(vec![
        Ld0U64,
        Scope(vec![
            TakeRef(s(0)),
            Scope(vec![TakeRef(s(1)), Scope(vec![TakeMut(s(2))])]),
        ]),
        Scope(vec![TakeMut(s(0))]),
    ]);
}

#[test]
fn test_write_to_mut_borrowed_value() {
    let e = run_err(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![TakeMut(s(0)), UAdd(three(0, 0, 1))]),
    ]);
    assert_lock_error(e, LockError::WriteButLocked);
}

#[test]
fn test_read_of_mut_borrowed_value() {
    let e = run_err(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![TakeMut(s(0)), UAdd(three(1, 0, 1))]),
    ]);
    assert_lock_error(e, LockError::ReadButMutLocked);
}

#[test]
fn test_shared_borrow_allows_reads_only() {
    let vm = run_ok(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![TakeRef(s(0)), UAdd(three(1, 0, 1))]),
    ]);
    assert_eq!(u64_at(&vm, 1), 1);

    let e = run_err(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![TakeRef(s(0)), UAdd(three(0, 0, 1))]),
    ]);
    assert_lock_error(e, LockError::WriteButLocked);
}

#[test]
fn test_shared_borrows_are_counted() {
    // the copy of the reference is dropped, the original still borrows the value
    let e = run_err(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![
            TakeRef(s(0)),
            Scope(vec![Mp(s(2))]),
            UAdd(three(0, 0, 1)),
        ]),
    ]);
    assert_lock_error(e, LockError::WriteButLocked);

    let vm = run_ok(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![TakeRef(s(0)), Scope(vec![Mp(s(2))])]),
        UAdd(three(0, 0, 1)),
    ]);
    assert_eq!(u64_at(&vm, 0), 1);
}

#[test]
fn test_mut_ref_is_moved_by_mp() {
    let e = run_err(vec![
        Ld0U64,
        Scope(vec![TakeMut(s(0)), Mp(s(1)), Mp(s(1))]),
    ]);
    assert!(matches!(e, VmError::UseOfMovedValue(StackRef(1))), "{e:?}");

    let e = run_err(vec![
        Ld0U64,
        Scope(vec![TakeMut(s(0)), Mp(s(1)), StartDeref(s(1))]),
    ]);
    assert!(matches!(e, VmError::UseOfMovedValue(StackRef(1))), "{e:?}");
}

#[test]
fn test_mv_moves_mut_ref() {
    let vm = run_ok(vec![
        Ld0U64, // 0: x
        Ld0U64, // 1: y
        Scope(vec![
            TakeMut(s(0)),                           // 2: &mut x
            Scope(vec![TakeMut(s(1)), Mv(s(2), s(3))]), // 2: &mut y
            // x was released when its reference was overwritten
            Scope(vec![TakeMut(s(0))]),
            // the moved-to reference is usable
            ld_u64(1),
            StartDeref(s(2)),
            UAdd(three(4, 4, 3)),
            EndDeref,
        ]),
    ]);
    assert_eq!(u64_at(&vm, 0), 0);
    assert_eq!(u64_at(&vm, 1), 1);
}

#[test]
fn test_popped_moved_ref_keeps_borrow() {
    let e = run_err(vec![
        Ld0U64,
        Ld0U64,
        Scope(vec![
            TakeMut(s(0)),
            Scope(vec![TakeMut(s(1)), Mv(s(2), s(3))]),
            // y is still mutably borrowed by @2
            Scope(vec![TakeRef(s(1))]),
        ]),
    ]);
    assert_lock_error(e, LockError::RefLockButMutLocked);
}

#[test]
fn test_ref_cannot_escape_its_value() {
    let e = run_err(vec![
        Ld0U64,
        Scope(vec![
            TakeMut(s(0)),
            Scope(vec![Ld0U64, Scope(vec![TakeMut(s(2)), Mv(s(1), s(3))])]),
        ]),
    ]);
    assert!(matches!(e, VmError::ReferenceEscapesScope(StackRef(1))), "{e:?}");
}

#[test]
fn test_array_index_out_of_bounds() {
    let e = run_err(vec![
        SArrCreate0(3, p(0)),
        ld_u64(3),
        Scope(vec![
            TakeMut(s(0)),
            Scope(vec![SArrMut {
                arr_mut: s(2),
                index: s(1),
            }]),
        ]),
    ]);
    assert!(matches!(e, VmError::IndexOutOfBounds { index: 3, len: 3 }), "{e:?}");
}

#[test]
fn test_array_element_borrows_are_exclusive() {
    let e = run_err(vec![
        SArrCreate0(3, p(0)),
        ld_u64(0),
        ld_u64(1),
        Scope(vec![
            TakeMut(s(0)),
            Scope(vec![
                SArrMut {
                    arr_mut: s(3),
                    index: s(1),
                },
                // a different element can be borrowed
                SArrMut {
                    arr_mut: s(3),
                    index: s(2),
                },
                SArrGet {
                    arr_ref: s(3),
                    index: s(1),
                },
            ]),
        ]),
    ]);
    assert_lock_error(e, LockError::RefLockButMutLocked);
}

#[test]
fn test_element_borrows_block_array_ref() {
    let e = run_err(vec![
        SArrCreate0(3, p(0)),
        ld_u64(0),
        Scope(vec![
            TakeMut(s(0)),
            Scope(vec![
                SArrMut {
                    arr_mut: s(2),
                    index: s(1),
                },
                StartDeref(s(2)),
            ]),
        ]),
    ]);
    assert_lock_error(e, LockError::ElementsBorrowed);
}

#[test]
fn test_array_element_write() {
    let vm = run_ok(vec![
        SArrCreate0(3, p(0)),
        ld_u64(1),
        Scope(vec![
            TakeMut(s(0)),
            Scope(vec![
                SArrMut {
                    arr_mut: s(2),
                    index: s(1),
                },
                StartDeref(s(3)),
                UAdd(three(4, 4, 1)),
                EndDeref,
            ]),
            // the element borrow was released with its reference
            StartDeref(s(2)),
            EndDeref,
        ]),
    ]);
    assert_eq!(data_at(&vm, 0), vec![0, 1, 0]);
}

#[test]
fn test_shared_element_refs_through_copied_array_refs() {
    run_ok(vec![
        SArrCreate0(3, p(0)),
        ld_u64(0),
        Scope(vec![
            TakeRef(s(0)),
            Mp(s(2)),
            Scope(vec![
                SArrGet {
                    arr_ref: s(2),
                    index: s(1),
                },
                SArrGet {
                    arr_ref: s(3),
                    index: s(1),
                },
                Mp(s(4)),
            ]),
        ]),
    ]);
}

#[test]
fn test_deref_writes_back_to_pointee() {
    // the string occupies two stack values, so the indexes of values and data differ
    let vm = run_ok(vec![
        LdSS(p(5)),
        Ld0U64,
        ld_u64(1),
        Scope(vec![
            TakeMut(s(1)),
            StartDeref(s(3)),
            UAdd(three(4, 4, 2)),
            EndDeref,
        ]),
    ]);
    assert_eq!(u64_at(&vm, 1), 1);
    assert_eq!(data_at(&vm, 0)[1], STR.len() as u64);
}

#[test]
fn test_scope_cannot_end_during_deref() {
    let e = run_err(vec![
        Ld0U64,
        Scope(vec![TakeMut(s(0)), Scope(vec![StartDeref(s(1))])]),
    ]);
    assert!(matches!(e, VmError::ScopeEndedDuringDeref), "{e:?}");
}

#[test]
fn test_deref_of_ref_to_ref() {
    let e = run_err(vec![
        Ld0U64,
        Scope(vec![TakeRef(s(0)), Scope(vec![TakeRef(s(1)), StartDeref(s(2))])]),
    ]);
    assert!(matches!(e, VmError::DerefOfBorrowingValue(StackRef(2))), "{e:?}");
}

#[test]
fn test_ended_deref_is_dead() {
    let e = run_err(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![
            TakeMut(s(0)),
            StartDeref(s(2)),
            EndDeref,
            UAdd(three(1, 3, 1)),
        ]),
    ]);
    assert!(matches!(e, VmError::UseOfMovedValue(StackRef(3))), "{e:?}");
}

#[test]
fn test_pool_loads_need_single_value_types() {
    for t in [p(4), p(6)] {
        let e = run_err(vec![LdTyped0 { type_location: t }]);
        assert!(matches!(e, VmError::ConstantPoolError), "{e:?}");
    }
    let e = run_err(vec![LDType {
        type_location: p(4),
        value_location: p(1),
    }]);
    assert!(matches!(e, VmError::ConstantPoolError), "{e:?}");
    let e = run_err(vec![LDType {
        type_location: p(7),
        value_location: p(8),
    }]);
    assert!(matches!(e, VmError::ConstantPoolError), "{e:?}");
}

#[test]
fn test_unary_ops_consume_their_operands() {
    // a wrong instruction size would misalign everything after it
    let vm = run_ok(vec![
        LdFalse,
        LdFalse,
        BNot(two(1, 0)),
        Ld0U64,
        Mp(s(2)),
        LNot(two(3, 2)),
        ld_u64(1),
    ]);
    let not_false: bool = vm.single_stack_data(s(1)).unwrap().into_primitive();
    assert!(not_false);
    assert_eq!(u64_at(&vm, 3), u64::MAX);
    assert_eq!(u64_at(&vm, 4), 1);
}
