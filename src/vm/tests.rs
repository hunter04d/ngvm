//! Tests of the memory model: borrows, moves, scopes, derefs and function calls

use crate::code::refs::{p, s, three, two, StackRef};
use crate::error::{VmContextError, VmError};
use crate::model::Opcode::{self, *};
use crate::stack::data::{IntoPrimitive, StackData};
use crate::types::PrimitiveType::*;
use crate::types::checker::TypeError;
use crate::types::{PointedType, RefKind, VmType};
use crate::vm::lock::{LockError, ValueLock};
use crate::vm::refs::LocatedRef;
use crate::{Code, Constant, ConstantPool, Function, Module, NativeCall, Signature, Vm};

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
    ConstantPool::new(constants())
}

fn constants() -> Vec<Constant> {
    vec![
        U64.into(),
        0u64.into(),
        1u64.into(),
        3u64.into(),
        SStr.into(),
        STR.into(),
        Never.into(),
        Char.into(),
        Constant::from(0xD800u32),
    ]
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
    ok(run(code))
}

fn run_err(code: Vec<Opcode>) -> VmError {
    err(run(code))
}

fn ok((vm, result): (Vm, Result<(), VmContextError>)) -> Vm {
    if let Err(e) = result {
        panic!("program failed: {:?}", e);
    }
    assert!(vm.frames.is_empty());
    assert!(vm.derefs.is_empty());
    assert!(vm.transient_refs.is_empty());
    for meta in &vm.stack_metadata {
        assert_eq!(meta.lock, ValueLock::None, "{:?}", meta);
    }
    vm
}

fn err((_, result): (Vm, Result<(), VmContextError>)) -> VmError {
    result.expect_err("program must fail").error
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
            Add(three(3, 3, 2)),
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
        Scope(vec![TakeMut(s(0)), Add(three(0, 0, 1))]),
    ]);
    assert_lock_error(e, LockError::WriteButLocked);
}

#[test]
fn test_read_of_mut_borrowed_value() {
    let e = run_err(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![TakeMut(s(0)), Add(three(1, 0, 1))]),
    ]);
    assert_lock_error(e, LockError::ReadButMutLocked);
}

#[test]
fn test_shared_borrow_allows_reads_only() {
    let vm = run_ok(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![TakeRef(s(0)), Add(three(1, 0, 1))]),
    ]);
    assert_eq!(u64_at(&vm, 1), 1);

    let e = run_err(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![TakeRef(s(0)), Add(three(0, 0, 1))]),
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
            Add(three(0, 0, 1)),
        ]),
    ]);
    assert_lock_error(e, LockError::WriteButLocked);

    let vm = run_ok(vec![
        Ld0U64,
        ld_u64(1),
        Scope(vec![TakeRef(s(0)), Scope(vec![Mp(s(2))])]),
        Add(three(0, 0, 1)),
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
            Add(three(4, 4, 3)),
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
fn test_ref_is_two_stack_values() {
    let (vm, result) = run(vec![Ld0U64, StartScope, TakeRef(s(0)), TraceStackValue(s(1))]);
    result.unwrap();
    assert_eq!(vm.stack.len(), 3);
    assert_eq!(vm.stack_data(s(1)).unwrap().len(), 2);
    let (located, _) = vm.locate_ref(s(1)).unwrap();
    assert_eq!(located, LocatedRef::Stack(0));
}

#[test]
fn test_element_ref_moves_into_stack_ref() {
    let vm = run_ok(vec![
        SArrCreate0(3, p(0)), // 0
        ld_u64(1),            // 1
        Ld0U64,               // 2: x
        Scope(vec![
            TakeMut(s(0)), // 3: &mut arr
            TakeMut(s(2)), // 4: &mut x
            Scope(vec![
                SArrMut {
                    arr_mut: s(3),
                    index: s(1),
                },
                // 4: &mut arr[1], x is released
                Mv(s(4), s(5)),
            ]),
            Scope(vec![TakeMut(s(2))]),
            StartDeref(s(4)),
            Add(three(5, 5, 1)),
            EndDeref,
        ]),
    ]);
    assert_eq!(data_at(&vm, 0), vec![0, 1, 0]);
    assert_eq!(u64_at(&vm, 2), 0);
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
                Add(three(4, 4, 1)),
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
            Add(three(4, 4, 2)),
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
            Add(three(1, 3, 1)),
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

/// Index of the first function in the pool of [`run_fns`]
const FNS: usize = 9;

/// A function of the module of [`run_fns`]
struct TestFn {
    name: &'static str,
    signature: Signature,
    code: Vec<Opcode>,
}

fn test_fn(
    name: &'static str,
    params: Vec<VmType>,
    ret: impl Into<VmType>,
    code: Vec<Opcode>,
) -> TestFn {
    multi_fn(name, params, vec![ret.into()], code)
}

fn multi_fn(
    name: &'static str,
    params: Vec<VmType>,
    returns: Vec<VmType>,
    code: Vec<Opcode>,
) -> TestFn {
    TestFn {
        name,
        signature: Signature::with_returns(params, returns).expect("at least one return value"),
        code,
    }
}

/// Module with `fns`, the function at index `i` is called with `Call(p(FNS + i))`,
/// `extra` constants follow the functions
fn module(fns: Vec<TestFn>, extra: Vec<Constant>) -> Module {
    let mut constants = constants();
    constants.extend(fns.iter().map(|f| Constant::function(f.name)));
    constants.extend(extra);
    let mut module = Module::new(ConstantPool::new(constants));
    for f in fns {
        let code = Code::from_model(&f.code).expect("valid model");
        module.add_fn(f.name.into(), Function::new(f.signature, code));
    }
    module
}

fn run_fns(fns: Vec<TestFn>, code: Vec<Opcode>) -> (Vm, Result<(), VmContextError>) {
    run_in(Vm::with_module(module(fns, vec![])), code)
}

fn run_in(mut vm: Vm, code: Vec<Opcode>) -> (Vm, Result<(), VmContextError>) {
    let code = Code::from_model(&code).expect("valid model");
    let result = code.interpret(&mut vm);
    (vm, result)
}

fn u64_t() -> VmType {
    U64.into()
}

fn ref_t(kind: RefKind, t: impl Into<VmType>) -> VmType {
    PointedType::reference(t, kind).into()
}

/// `inc(x: u64) -> u64`
fn inc() -> TestFn {
    test_fn(
        "inc",
        vec![u64_t()],
        U64,
        vec![ld_u64(1), Add(three(0, 0, 1)), Ret(s(0))],
    )
}

#[test]
fn test_call_returns_value() {
    let vm = ok(run_fns(vec![inc()], vec![ld_u64(3), Mp(s(0)), Call(p(FNS))]));
    // the argument and the locals of the function are gone, the result took their place
    assert_eq!(vm.stack_metadata.len(), 2);
    assert_eq!(u64_at(&vm, 0), 3);
    assert_eq!(u64_at(&vm, 1), 4);
    assert_eq!(vm.current_cycle(), 1);
}

#[test]
fn test_call_continues_after_return() {
    let vm = ok(run_fns(
        vec![inc()],
        vec![ld_u64(0), Call(p(FNS)), Call(p(FNS)), Call(p(FNS)), ld_u64(1)],
    ));
    assert_eq!(u64_at(&vm, 0), 3);
    assert_eq!(u64_at(&vm, 1), 1);
}

#[test]
fn test_nested_and_recursive_calls() {
    // fact(n: u64) -> u64
    let fact = test_fn(
        "fact",
        vec![u64_t()],
        U64,
        vec![
            ld_u64(1),       // 1
            LdFalse,         // 2
            Le(three(2, 0, 1)),
            JC {
                label: 0,
                cond: s(2),
            },
            Mp(s(0)),        // 3: n
            Sub(three(3, 3, 1)),
            Call(p(FNS)),    // 3: fact(n - 1)
            Mul(three(3, 3, 0)),
            Ret(s(3)),
            Label(0),
            Ret(s(1)),
        ],
    );
    // calls fact(fact(3))
    let fact_of_fact = test_fn(
        "fact_of_fact",
        vec![u64_t()],
        U64,
        vec![Call(p(FNS)), Call(p(FNS)), Ret(s(0))],
    );
    let vm = ok(run_fns(vec![fact, fact_of_fact], vec![ld_u64(3), Call(p(FNS + 1))]));
    assert_eq!(u64_at(&vm, 0), 720);
}

#[test]
fn test_deep_recursion() {
    // count(n: u64) -> u64, counts down to 0 and returns the number of calls
    let count = test_fn(
        "count",
        vec![u64_t()],
        U64,
        vec![
            ld_u64(0),       // 1
            LdFalse,         // 2
            Eq(three(2, 0, 1)),
            JC {
                label: 0,
                cond: s(2),
            },
            ld_u64(1),       // 3
            Mp(s(0)),        // 4
            Sub(three(4, 4, 3)),
            Call(p(FNS)),
            Add(three(4, 4, 3)),
            Ret(s(4)),
            Label(0),
            Ret(s(1)),
        ],
    );
    let vm = Vm::with_module(module(vec![count], vec![100_000u64.into()]));
    let vm = ok(run_in(
        vm,
        vec![
            LDType {
                type_location: p(0),
                value_location: p(FNS + 1),
            },
            Call(p(FNS)),
        ],
    ));
    assert_eq!(u64_at(&vm, 0), 100_000);
}

/// `set_one(x: &mut u64) -> ()`
fn set_one() -> TestFn {
    test_fn(
        "set_one",
        vec![ref_t(RefKind::Mut, U64)],
        Unit,
        vec![
            StartDeref(s(0)), // 1
            ld_u64(1),        // 2
            Add(three(1, 1, 2)),
            EndDeref,
            LdUnit,
            Ret(s(3)),
        ],
    )
}

/// `id(x: &u64) -> &u64`
fn id() -> TestFn {
    test_fn(
        "id",
        vec![ref_t(RefKind::Ref, U64)],
        ref_t(RefKind::Ref, U64),
        vec![Ret(s(0))],
    )
}

#[test]
fn test_call_mutates_through_mut_ref_argument() {
    let vm = ok(run_fns(
        vec![set_one()],
        vec![Ld0U64, Scope(vec![TakeMut(s(0)), Call(p(FNS))])],
    ));
    assert_eq!(u64_at(&vm, 0), 1);
}

#[test]
fn test_element_ref_is_passed_as_ref() {
    let vm = ok(run_fns(
        vec![set_one()],
        vec![
            SArrCreate0(3, p(0)),
            ld_u64(1),
            Scope(vec![
                TakeMut(s(0)),
                Scope(vec![
                    SArrMut {
                        arr_mut: s(2),
                        index: s(1),
                    },
                    Call(p(FNS)),
                ]),
            ]),
        ],
    ));
    assert_eq!(data_at(&vm, 0), vec![0, 1, 0]);

    // the returned element reference still borrows the element through the array reference
    let vm = ok(run_fns(
        vec![id()],
        vec![
            SArrCreate0(3, p(0)),
            ld_u64(1),
            Scope(vec![
                TakeRef(s(0)),
                Scope(vec![
                    SArrGet {
                        arr_ref: s(2),
                        index: s(1),
                    },
                    Call(p(FNS)),
                    StartDeref(s(3)),
                    EndDeref,
                ]),
            ]),
        ],
    ));
    assert_eq!(vm.stack_metadata.len(), 2);
    let e = err(run_fns(
        vec![id()],
        vec![
            SArrCreate0(3, p(0)),
            ld_u64(1),
            Scope(vec![
                TakeRef(s(0)),
                Scope(vec![
                    SArrGet {
                        arr_ref: s(2),
                        index: s(1),
                    },
                    Call(p(FNS)),
                    TakeRef(s(2)),
                ]),
            ]),
        ],
    ));
    assert_lock_error(e, LockError::ElementsBorrowed);

    // the kind of the reference still has to match
    let e = err(run_fns(
        vec![set_one()],
        vec![
            SArrCreate0(3, p(0)),
            ld_u64(1),
            Scope(vec![
                TakeRef(s(0)),
                Scope(vec![
                    SArrGet {
                        arr_ref: s(2),
                        index: s(1),
                    },
                    Call(p(FNS)),
                ]),
            ]),
        ],
    ));
    assert!(matches!(e, VmError::TypeError(_)), "{e:?}");
}

#[test]
fn test_returned_ref_keeps_borrow() {
    let vm = ok(run_fns(
        vec![id()],
        vec![
            ld_u64(3),
            Scope(vec![
                TakeRef(s(0)),
                Call(p(FNS)),
                StartDeref(s(1)),
                Mp(s(2)),
                EndDeref,
            ]),
        ],
    ));
    assert_eq!(vm.stack_metadata.len(), 1);

    let e = err(run_fns(
        vec![id()],
        vec![
            Ld0U64,
            ld_u64(1),
            Scope(vec![TakeRef(s(0)), Call(p(FNS)), Add(three(0, 0, 1))]),
        ],
    ));
    assert_lock_error(e, LockError::WriteButLocked);
}

#[test]
fn test_ref_to_frame_cannot_be_returned() {
    let local = test_fn(
        "local",
        vec![],
        ref_t(RefKind::Ref, U64),
        vec![Ld0U64, StartScope, TakeRef(s(0)), Ret(s(1))],
    );
    let e = err(run_fns(vec![local], vec![Call(p(FNS))]));
    assert!(matches!(e, VmError::ReferenceEscapesScope(StackRef(1))), "{e:?}");

    let arg = test_fn(
        "arg",
        vec![u64_t()],
        ref_t(RefKind::Mut, U64),
        vec![TakeMut(s(0)), Ret(s(1))],
    );
    let e = err(run_fns(vec![arg], vec![Ld0U64, Call(p(FNS))]));
    assert!(matches!(e, VmError::ReferenceEscapesScope(StackRef(1))), "{e:?}");
}

#[test]
fn test_call_checks_arguments() {
    let e = err(run_fns(vec![inc()], vec![LdTrue, Call(p(FNS))]));
    assert!(matches!(e, VmError::TypeError(ref e) if e.len() == 1), "{e:?}");

    let e = err(run_fns(vec![inc()], vec![Call(p(FNS))]));
    assert!(
        matches!(e, VmError::NotEnoughArguments { expected: 1, got: 0 }),
        "{e:?}"
    );

    // the values of the caller cannot be taken as arguments
    let no_args = test_fn("no_args", vec![], U64, vec![Call(p(FNS))]);
    let e = err(run_fns(vec![inc(), no_args], vec![Ld0U64, Call(p(FNS + 1))]));
    assert!(
        matches!(e, VmError::NotEnoughArguments { expected: 1, got: 0 }),
        "{e:?}"
    );

    let e = err(run_fns(
        vec![test_fn(
            "two",
            vec![ref_t(RefKind::Mut, U64), ref_t(RefKind::Mut, U64)],
            Unit,
            vec![LdUnit, Ret(s(2))],
        )],
        vec![Ld0U64, Scope(vec![TakeMut(s(0)), Mp(s(1)), Call(p(FNS))])],
    ));
    assert!(matches!(e, VmError::UseOfMovedValue(StackRef(1))), "{e:?}");

    let e = err(run_fns(
        vec![inc()],
        vec![
            Ld0U64,
            Scope(vec![TakeMut(s(0)), StartDeref(s(1)), Call(p(FNS))]),
        ],
    ));
    assert!(matches!(e, VmError::DerefPassedToCall), "{e:?}");
}

#[test]
fn test_ret_checks_value() {
    let wrong = test_fn("wrong", vec![], U64, vec![LdTrue, Ret(s(0))]);
    let e = err(run_fns(vec![wrong], vec![Call(p(FNS))]));
    assert!(matches!(e, VmError::TypeError(_)), "{e:?}");

    let borrowed = test_fn(
        "borrowed",
        vec![],
        U64,
        vec![Ld0U64, StartScope, TakeMut(s(0)), Ret(s(0))],
    );
    let e = err(run_fns(vec![borrowed], vec![Call(p(FNS))]));
    assert_lock_error(e, LockError::ReadButMutLocked);

    let in_deref = test_fn(
        "in_deref",
        vec![ref_t(RefKind::Mut, U64)],
        Unit,
        vec![StartDeref(s(0)), LdUnit, Ret(s(2))],
    );
    let e = err(run_fns(
        vec![in_deref],
        vec![Ld0U64, Scope(vec![TakeMut(s(0)), Call(p(FNS))])],
    ));
    assert!(matches!(e, VmError::ScopeEndedDuringDeref), "{e:?}");

    let e = run_err(vec![Ld0U64, Ret(s(0))]);
    assert!(matches!(e, VmError::RetOutsideFunction), "{e:?}");

    let no_ret = test_fn("no_ret", vec![], U64, vec![Ld0U64]);
    let e = err(run_fns(vec![no_ret], vec![Call(p(FNS))]));
    assert!(matches!(e, VmError::MissingReturn), "{e:?}");
}

#[test]
fn test_function_is_confined_to_its_frame() {
    // the values of the caller are not visible
    let read = test_fn("read", vec![], U64, vec![Ret(s(0))]);
    let e = err(run_fns(vec![read], vec![Ld0U64, Call(p(FNS))]));
    assert!(matches!(e, VmError::BadVmState), "{e:?}");

    // the scope of the caller cannot be ended
    let end_scope = test_fn("end_scope", vec![u64_t()], U64, vec![EndScope, Ret(s(0))]);
    let e = err(run_fns(
        vec![end_scope],
        vec![Ld0U64, Scope(vec![Ld0U64, Call(p(FNS))])],
    ));
    assert!(matches!(e, VmError::BadVmState), "{e:?}");

    // the deref of the caller cannot be ended
    let end_deref = test_fn("end_deref", vec![], Unit, vec![EndDeref, LdUnit, Ret(s(0))]);
    let e = err(run_fns(
        vec![end_deref],
        vec![
            Ld0U64,
            Scope(vec![TakeMut(s(0)), StartDeref(s(1)), Call(p(FNS))]),
        ],
    ));
    assert!(matches!(e, VmError::BadVmState), "{e:?}");

    // scopes started by the function end with it
    let scoped = test_fn(
        "scoped",
        vec![u64_t()],
        U64,
        vec![StartScope, StartScope, TakeRef(s(0)), Ret(s(0))],
    );
    let vm = ok(run_fns(vec![scoped], vec![ld_u64(3), Call(p(FNS))]));
    assert_eq!(u64_at(&vm, 0), 3);
    assert_eq!(vm.current_cycle(), 1);
}

#[test]
fn test_call_into_another_module() {
    let new_vm = || {
        // the function uses the constant pool of its own module
        let mut math = Module::new(ConstantPool::new(vec![U64.into(), 10u64.into()]));
        let add_ten = vec![
            LDType {
                type_location: p(0),
                value_location: p(1),
            },
            Add(three(0, 0, 1)),
            Ret(s(0)),
        ];
        let code = Code::from_model(&add_ten).unwrap();
        math.add_fn(
            "add_ten".into(),
            Function::new(Signature::new(vec![u64_t()], U64), code),
        );
        let fns = vec![
            Constant::module_function("math", "add_ten"),
            Constant::module_function("math", "missing"),
            Constant::module_function("missing", "add_ten"),
        ];
        let mut vm = Vm::with_module(module(vec![], fns));
        vm.add_module("math", math).unwrap();
        vm
    };
    // the pool of the caller is used again after the return
    let vm = ok(run_in(new_vm(), vec![ld_u64(1), Call(p(FNS)), ld_u64(3)]));
    assert_eq!(u64_at(&vm, 0), 11);
    assert_eq!(u64_at(&vm, 1), 3);

    let e = err(run_in(new_vm(), vec![ld_u64(1), Call(p(FNS + 1))]));
    assert!(
        matches!(
            e,
            VmError::UnknownFunction { ref module, ref name } if module == "math" && name == "missing"
        ),
        "{e:?}"
    );
    let e = err(run_in(new_vm(), vec![ld_u64(1), Call(p(FNS + 2))]));
    assert!(
        matches!(
            e,
            VmError::UnknownFunction { ref module, ref name } if module == "missing" && name == "add_ten"
        ),
        "{e:?}"
    );
}

#[test]
fn test_call_decodes() {
    let code = Code::from_model(&[
        Ld0U64,
        Call(p(FNS)),
        Ret(s(0)),
        RetN {
            first: s(0),
            count: 2,
        },
    ])
    .unwrap();
    let decoded = code.decode();
    assert!(decoded.is_full);
    assert_eq!(decoded.opcodes.len(), 4);
}

/// `div_rem(a: u64, b: u64) -> (u64, u64)`
fn div_rem() -> TestFn {
    multi_fn(
        "div_rem",
        vec![u64_t(), u64_t()],
        vec![u64_t(), u64_t()],
        vec![
            Ld0U64, // 2
            Ld0U64, // 3
            Div(three(2, 0, 1)),
            Rem(three(3, 0, 1)),
            RetN {
                first: s(2),
                count: 2,
            },
        ],
    )
}

#[test]
fn test_return_multiple_values() {
    let vm = Vm::with_module(module(vec![div_rem()], vec![17u64.into(), 5u64.into()]));
    let ld = |i| LDType {
        type_location: p(0),
        value_location: p(FNS + i),
    };
    let vm = ok(run_in(vm, vec![ld_u64(1), ld(1), ld(2), Call(p(FNS))]));
    assert_eq!(vm.stack_metadata.len(), 3);
    assert_eq!(u64_at(&vm, 0), 1);
    assert_eq!(u64_at(&vm, 1), 3);
    assert_eq!(u64_at(&vm, 2), 2);
}

#[test]
fn test_at_least_one_value_is_returned() {
    assert!(Signature::with_returns(vec![u64_t()], vec![]).is_none());

    let nothing = test_fn(
        "nothing",
        vec![],
        U64,
        vec![
            Ld0U64,
            RetN {
                first: s(0),
                count: 0,
            },
        ],
    );
    let e = err(run_fns(vec![nothing], vec![Call(p(FNS))]));
    assert!(matches!(e, VmError::InvalidBytecode), "{e:?}");
}

#[test]
fn test_return_multiple_refs() {
    // swap(a: &mut u64, b: &mut u64) -> (&mut u64, &mut u64)
    let swap = multi_fn(
        "swap",
        vec![ref_t(RefKind::Mut, U64), ref_t(RefKind::Mut, U64)],
        vec![ref_t(RefKind::Mut, U64), ref_t(RefKind::Mut, U64)],
        vec![
            Mp(s(1)), // 2
            Mp(s(0)), // 3
            RetN {
                first: s(2),
                count: 2,
            },
        ],
    );
    let vm = ok(run_fns(
        vec![swap],
        vec![
            Ld0U64, // 0: x
            Ld0U64, // 1: y
            Scope(vec![
                TakeMut(s(0)), // 2
                TakeMut(s(1)), // 3
                // 2: &mut y, 3: &mut x
                Call(p(FNS)),
                StartDeref(s(2)), // 4
                ld_u64(1),        // 5
                Add(three(4, 4, 5)),
                EndDeref,
            ]),
        ],
    ));
    assert_eq!(u64_at(&vm, 0), 0);
    assert_eq!(u64_at(&vm, 1), 1);
}

#[test]
fn test_return_checks_all_values() {
    let two = |code| multi_fn("two", vec![], vec![u64_t(), u64_t()], code);
    let call = |f| err(run_fns(vec![f], vec![Call(p(FNS))]));

    let e = call(two(vec![Ld0U64, Ret(s(0))]));
    assert!(
        matches!(e, VmError::WrongReturnCount { expected: 2, got: 1 }),
        "{e:?}"
    );
    let e = call(test_fn(
        "one",
        vec![],
        U64,
        vec![
            Ld0U64,
            Ld0U64,
            RetN {
                first: s(0),
                count: 2,
            },
        ],
    ));
    assert!(
        matches!(e, VmError::WrongReturnCount { expected: 1, got: 2 }),
        "{e:?}"
    );

    let e = call(two(vec![
        Ld0U64,
        LdTrue,
        RetN {
            first: s(0),
            count: 2,
        },
    ]));
    match e {
        VmError::TypeError(errors) => match errors.as_slice() {
            [TypeError::NotEquals(t, _)] => assert_eq!(t.tag.to_string(), "ret1"),
            errors => panic!("{errors:?}"),
        },
        e => panic!("{e:?}"),
    }

    let e = call(multi_fn(
        "escapes",
        vec![],
        vec![u64_t(), ref_t(RefKind::Ref, U64)],
        vec![
            Ld0U64,
            StartScope,
            TakeRef(s(0)),
            RetN {
                first: s(0),
                count: 2,
            },
        ],
    ));
    assert!(matches!(e, VmError::ReferenceEscapesScope(StackRef(1))), "{e:?}");

    // the values must be in the frame
    let e = call(two(vec![
        Ld0U64,
        RetN {
            first: s(0),
            count: 2,
        },
    ]));
    assert!(matches!(e, VmError::BadVmState), "{e:?}");
}


/// Module with the native functions `natives` followed by the bytecode functions `fns`,
/// the function at index `i` is called with `Call(p(FNS + i))`
fn native_module(natives: Vec<(&'static str, Function)>, fns: Vec<TestFn>) -> Module {
    let mut constants = constants();
    constants.extend(natives.iter().map(|(name, _)| Constant::function(name)));
    constants.extend(fns.iter().map(|f| Constant::function(f.name)));
    let mut module = Module::new(ConstantPool::new(constants));
    for (name, f) in natives {
        module.add_fn(name.into(), f);
    }
    for f in fns {
        let code = Code::from_model(&f.code).expect("valid model");
        module.add_fn(f.name.into(), Function::new(f.signature, code));
    }
    module
}

fn run_natives(
    natives: Vec<(&'static str, Function)>,
    code: Vec<Opcode>,
) -> (Vm, Result<(), VmContextError>) {
    run_in(Vm::with_module(native_module(natives, vec![])), code)
}

/// `add(a: u64, b: u64) -> u64`
fn native_add() -> (&'static str, Function) {
    let f = Function::native(Signature::new(vec![u64_t(), u64_t()], U64), |call| {
        let sum = call.arg::<u64>(0)? + call.arg::<u64>(1)?;
        call.ret(sum);
        Ok(())
    });
    ("add", f)
}

#[test]
fn test_native_call_returns_value() {
    let vm = ok(run_natives(
        vec![native_add()],
        vec![ld_u64(1), ld_u64(3), Call(p(FNS)), ld_u64(1)],
    ));
    // the arguments are gone, the result took their place and execution continued
    assert_eq!(vm.stack_metadata.len(), 2);
    assert_eq!(u64_at(&vm, 0), 4);
    assert_eq!(u64_at(&vm, 1), 1);
    assert_eq!(vm.current_cycle(), 1);
}

#[test]
fn test_native_called_from_bytecode_function() {
    // add_three(x: u64) -> u64 { add(add(x, 1), 1) + 1 }
    let add_three = test_fn(
        "add_three",
        vec![u64_t()],
        U64,
        vec![
            ld_u64(1),
            Call(p(FNS)),
            ld_u64(1),
            Call(p(FNS)),
            ld_u64(1),
            Add(three(0, 0, 1)),
            Ret(s(0)),
        ],
    );
    let vm = Vm::with_module(native_module(vec![native_add()], vec![add_three]));
    let vm = ok(run_in(vm, vec![ld_u64(3), Call(p(FNS + 1))]));
    assert_eq!(vm.stack_metadata.len(), 1);
    assert_eq!(u64_at(&vm, 0), 6);
}

#[test]
fn test_native_keeps_state() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&log);
    let print = Function::native(Signature::new(vec![u64_t()], Unit), move |call| {
        sink.borrow_mut().push(call.arg::<u64>(0)?);
        call.ret(());
        Ok(())
    });
    ok(run_natives(
        vec![("print", print)],
        vec![ld_u64(1), Call(p(FNS)), ld_u64(3), Call(p(FNS))],
    ));
    assert_eq!(*log.borrow(), vec![1, 3]);
}

#[test]
fn test_native_returns_multiple_values() {
    let div_rem = Function::native(
        Signature::with_returns(vec![u64_t(), u64_t()], vec![u64_t(), u64_t()]).unwrap(),
        |call| {
            let (a, b) = (call.arg::<u64>(0)?, call.arg::<u64>(1)?);
            if b == 0 {
                return Err(VmError::Native("division by zero".into()));
            }
            call.ret(a / b);
            call.ret(a % b);
            Ok(())
        },
    );
    let vm = Vm::with_module(native_module(vec![("div_rem", div_rem)], vec![]));
    let vm = ok(run_in(vm, vec![ld_u64(3), ld_u64(3), Call(p(FNS))]));
    assert_eq!(vm.stack_metadata.len(), 2);
    assert_eq!(u64_at(&vm, 0), 1);
    assert_eq!(u64_at(&vm, 1), 0);
}

#[test]
fn test_native_error_ends_execution() {
    let fail = Function::native(Signature::new(vec![], U64), |_| {
        Err(VmError::Native("failed".into()))
    });
    let e = err(run_natives(vec![("fail", fail)], vec![Call(p(FNS))]));
    assert!(matches!(e, VmError::Native(ref e) if e.to_string() == "failed"), "{e:?}");
}

#[test]
fn test_native_return_is_checked() {
    let call = |returns: Vec<VmType>, f: fn(&mut NativeCall<'_>) -> crate::vm::Result<()>| {
        let native = Function::native(Signature::with_returns(vec![], returns).unwrap(), f);
        err(run_natives(vec![("native", native)], vec![Call(p(FNS))]))
    };

    let e = call(vec![u64_t()], |_| Ok(()));
    assert!(
        matches!(e, VmError::WrongReturnCount { expected: 1, got: 0 }),
        "{e:?}"
    );
    let e = call(vec![u64_t()], |call| {
        call.ret(1u64);
        call.ret(2u64);
        Ok(())
    });
    assert!(
        matches!(e, VmError::WrongReturnCount { expected: 1, got: 2 }),
        "{e:?}"
    );
    let e = call(vec![u64_t(), u64_t()], |call| {
        call.ret(1u64);
        call.ret(true);
        Ok(())
    });
    match e {
        VmError::TypeError(errors) => match errors.as_slice() {
            [TypeError::NotEquals(t, _)] => assert_eq!(t.tag.to_string(), "ret1"),
            errors => panic!("{errors:?}"),
        },
        e => panic!("{e:?}"),
    }
}

#[test]
fn test_native_arguments_are_checked() {
    use std::cell::Cell;
    use std::rc::Rc;

    // the arguments are checked against the signature before the function runs
    let called = Rc::new(Cell::new(false));
    let flag = Rc::clone(&called);
    let native = Function::native(Signature::new(vec![u64_t()], U64), move |call| {
        flag.set(true);
        call.ret_arg(0)
    });
    let e = err(run_natives(vec![("native", native)], vec![LdTrue, Call(p(FNS))]));
    assert!(matches!(e, VmError::TypeError(_)), "{e:?}");
    assert!(!called.get());

    // the arguments are read with their own type
    let native = Function::native(Signature::new(vec![u64_t()], U64), |call| {
        let v: i64 = call.arg(0)?;
        call.ret(v as u64);
        Ok(())
    });
    let e = err(run_natives(vec![("native", native)], vec![Ld0U64, Call(p(FNS))]));
    match e {
        VmError::TypeError(errors) => match errors.as_slice() {
            [TypeError::NotEquals(t, _)] => assert_eq!(t.tag.to_string(), "arg0"),
            errors => panic!("{errors:?}"),
        },
        e => panic!("{e:?}"),
    }

    // only the arguments can be read, not the values of the caller or the returned ones
    let native = Function::native(Signature::new(vec![u64_t()], U64), |call| {
        call.ret(0u64);
        let v: u64 = call.arg(1)?;
        call.ret(v);
        Ok(())
    });
    let e = err(run_natives(vec![("native", native)], vec![Ld0U64, Ld0U64, Call(p(FNS))]));
    assert!(
        matches!(e, VmError::NoSuchArgument { index: 1, count: 1 }),
        "{e:?}"
    );
}

/// `set(x: &mut u64, v: u64) -> ()`
fn native_set() -> (&'static str, Function) {
    let params = vec![ref_t(RefKind::Mut, U64), u64_t()];
    let f = Function::native(Signature::new(params, Unit), |call| {
        let v = call.arg::<u64>(1)?;
        call.write(0, v)?;
        call.ret(());
        Ok(())
    });
    ("set", f)
}

/// `get(x: &u64) -> u64`
fn native_get() -> (&'static str, Function) {
    let f = Function::native(Signature::new(vec![ref_t(RefKind::Ref, U64)], U64), |call| {
        let v = call.read::<u64>(0)?;
        call.ret(v);
        Ok(())
    });
    ("get", f)
}

#[test]
fn test_native_reads_and_writes_through_refs() {
    let vm = ok(run_natives(
        vec![native_set(), native_get()],
        vec![
            Ld0U64,
            Scope(vec![TakeMut(s(0)), ld_u64(3), Call(p(FNS))]),
            Scope(vec![TakeRef(s(0)), Call(p(FNS + 1))]),
        ],
    ));
    assert_eq!(vm.stack_metadata.len(), 1);
    assert_eq!(u64_at(&vm, 0), 3);

    // through array element references
    let vm = ok(run_natives(
        vec![native_set(), native_get()],
        vec![
            SArrCreate0(3, p(0)),
            ld_u64(1),
            Scope(vec![
                TakeMut(s(0)),
                Scope(vec![
                    SArrMut {
                        arr_mut: s(2),
                        index: s(1),
                    },
                    ld_u64(3),
                    Call(p(FNS)),
                ]),
            ]),
            Scope(vec![
                TakeRef(s(0)),
                Scope(vec![
                    SArrGet {
                        arr_ref: s(2),
                        index: s(1),
                    },
                    Call(p(FNS + 1)),
                    // 3: the read element
                    Mv(s(1), s(3)),
                ]),
            ]),
        ],
    ));
    assert_eq!(data_at(&vm, 0), vec![0, 3, 0]);
    assert_eq!(u64_at(&vm, 1), 3);
}

#[test]
fn test_native_cannot_write_through_shared_ref() {
    let write = Function::native(Signature::new(vec![ref_t(RefKind::Ref, U64)], Unit), |call| {
        call.write(0, 1u64)?;
        call.ret(());
        Ok(())
    });
    let e = err(run_natives(
        vec![("write", write)],
        vec![Ld0U64, Scope(vec![TakeRef(s(0)), Call(p(FNS))])],
    ));
    assert!(matches!(e, VmError::TypeError(_)), "{e:?}");

    // nor through a reference of another type
    let write = Function::native(Signature::new(vec![ref_t(RefKind::Mut, U64)], Unit), |call| {
        call.write(0, 1i64)?;
        call.ret(());
        Ok(())
    });
    let e = err(run_natives(
        vec![("write", write)],
        vec![Ld0U64, Scope(vec![TakeMut(s(0)), Call(p(FNS))])],
    ));
    assert!(matches!(e, VmError::TypeError(_)), "{e:?}");
}

#[test]
fn test_native_returns_ref_argument() {
    // id(x: &mut u64) -> &mut u64
    let id = || {
        let t = ref_t(RefKind::Mut, U64);
        let f = Function::native(Signature::new(vec![t.clone()], t), |call| call.ret_arg(0));
        ("id", f)
    };
    let vm = ok(run_natives(
        vec![id(), native_set()],
        vec![
            Ld0U64,
            Scope(vec![
                TakeMut(s(0)),
                Call(p(FNS)),
                ld_u64(3),
                Call(p(FNS + 1)),
            ]),
        ],
    ));
    assert_eq!(u64_at(&vm, 0), 3);

    // the returned reference keeps the borrow
    let e = err(run_natives(
        vec![id()],
        vec![
            Ld0U64,
            ld_u64(1),
            Scope(vec![TakeMut(s(0)), Call(p(FNS)), Add(three(0, 0, 1))]),
        ],
    ));
    assert_lock_error(e, LockError::WriteButLocked);

    // a mutable reference is moved out of the argument
    let twice = Function::native(
        Signature::with_returns(
            vec![ref_t(RefKind::Mut, U64)],
            vec![ref_t(RefKind::Mut, U64), ref_t(RefKind::Mut, U64)],
        )
        .unwrap(),
        |call| {
            call.ret_arg(0)?;
            call.ret_arg(0)
        },
    );
    let e = err(run_natives(
        vec![("twice", twice)],
        vec![Ld0U64, Scope(vec![TakeMut(s(0)), Call(p(FNS))])],
    ));
    assert!(matches!(e, VmError::UseOfMovedValue(StackRef(0))), "{e:?}");

    let write_moved = Function::native(
        Signature::new(vec![ref_t(RefKind::Mut, U64)], ref_t(RefKind::Mut, U64)),
        |call| {
            call.ret_arg(0)?;
            call.write(0, 1u64)
        },
    );
    let e = err(run_natives(
        vec![("write_moved", write_moved)],
        vec![Ld0U64, Scope(vec![TakeMut(s(0)), Call(p(FNS))])],
    ));
    assert!(matches!(e, VmError::UseOfMovedValue(StackRef(0))), "{e:?}");
}

#[test]
fn test_native_in_another_module() {
    let mut math = Module::new(ConstantPool::new(vec![]));
    let (name, add) = native_add();
    math.add_fn(name.into(), add);
    let mut vm = Vm::with_module(module(vec![], vec![Constant::module_function("math", "add")]));
    vm.add_module("math", math).unwrap();
    // the pool of the caller is used again after the return
    let vm = ok(run_in(vm, vec![ld_u64(1), ld_u64(3), Call(p(FNS)), ld_u64(1)]));
    assert_eq!(u64_at(&vm, 0), 4);
    assert_eq!(u64_at(&vm, 1), 1);
}

#[test]
fn test_module_cannot_be_replaced() {
    let math = || {
        let mut math = Module::new(ConstantPool::new(vec![]));
        let (name, add) = native_add();
        math.add_fn(name.into(), add);
        math
    };
    let mut vm = Vm::with_module(module(vec![], vec![Constant::module_function("math", "add")]));
    vm.add_module("math", math()).unwrap();
    let e = vm.add_module("math", math()).map(|_| ()).unwrap_err();
    assert!(matches!(e, VmError::ModuleAlreadyLoaded(ref m) if m == "math"), "{e:?}");
    let e = vm.add_module("", math()).map(|_| ()).unwrap_err();
    assert!(matches!(e, VmError::ModuleAlreadyLoaded(ref m) if m.is_empty()), "{e:?}");

    // the loaded modules are still there
    let vm = ok(run_in(vm, vec![ld_u64(1), ld_u64(3), Call(p(FNS)), LdSS(p(5))]));
    assert_eq!(u64_at(&vm, 0), 4);
    assert_eq!(super::static_str(vm.stack_data(StackRef(1)).unwrap()).unwrap(), STR);
}

/// `len(s: SStr) -> u64`
fn native_len() -> (&'static str, Function) {
    let f = Function::native(Signature::new(vec![SStr.into()], U64), |call| {
        let len = call.arg_str(0)?.len();
        call.ret(len as u64);
        Ok(())
    });
    ("len", f)
}

#[test]
fn test_static_string_outlives_its_module_code() {
    // the string is loaded from the pool of another module and returned from there
    let mut strings = Module::new(ConstantPool::new(vec!["hello, world".into()]));
    let greet = Code::from_model(&[LdSS(p(0)), Ret(s(0))]).unwrap();
    strings.add_fn(
        "greet".into(),
        Function::new(Signature::new(vec![], SStr), greet),
    );
    let fns = vec![
        Constant::function("len"),
        Constant::module_function("strings", "greet"),
    ];
    let mut main = module(vec![], fns);
    let (name, len) = native_len();
    main.add_fn(name.into(), len);
    let mut vm = Vm::with_module(main);
    vm.add_module("strings", strings).unwrap();

    let vm = ok(run_in(
        vm,
        vec![Call(p(FNS + 1)), Mp(s(0)), Call(p(FNS)), Ld0U64, Mv(s(2), s(1))],
    ));
    assert_eq!(
        super::static_str(vm.stack_data(StackRef(0)).unwrap()).unwrap(),
        "hello, world"
    );
    assert_eq!(u64_at(&vm, 2), "hello, world".len() as u64);
}

#[test]
fn test_native_static_strings() {
    let sstr = || VmType::from(SStr);
    // concat_check(s: SStr, r: &SStr) -> (SStr, SStr, bool): returns s, a new string
    // and whether both strings are equal
    let native = Function::native(
        Signature::with_returns(
            vec![sstr(), ref_t(RefKind::Ref, SStr)],
            vec![sstr(), sstr(), Bool.into()],
        )
        .unwrap(),
        |call| {
            let equal = call.arg_str(0)? == call.read_str(1)?;
            call.ret_arg(0)?;
            call.ret_str("native");
            call.ret(equal);
            Ok(())
        },
    );
    let vm = ok(run_natives(
        vec![("native", native), native_len()],
        vec![
            LdSS(p(5)), // 0
            LdFalse,    // 1
            Ld0U64,     // 2
            Scope(vec![
                Mp(s(0)),      // 3
                TakeRef(s(0)), // 4
                Call(p(FNS)),
                // 3: s, 4: "native", 5: equal
                Mv(s(1), s(5)),
                Mv(s(0), s(4)),
                Mp(s(0)),
                Call(p(FNS + 1)),
                // 6: len("native")
                Mv(s(2), s(6)),
            ]),
        ],
    ));
    assert_eq!(vm.stack_metadata.len(), 3);
    assert_eq!(super::static_str(vm.stack_data(StackRef(0)).unwrap()).unwrap(), "native");
    let equal: bool = vm.single_stack_data(StackRef(1)).unwrap().into_primitive();
    assert!(equal);
    assert_eq!(u64_at(&vm, 2), "native".len() as u64);

    // a string is read with its own type
    let e = err(run_natives(vec![native_len()], vec![Ld0U64, Call(p(FNS))]));
    assert!(matches!(e, VmError::TypeError(_)), "{e:?}");
    let not_str = Function::native(Signature::new(vec![u64_t()], U64), |call| {
        let len = call.arg_str(0)?.len();
        call.ret(len as u64);
        Ok(())
    });
    let e = err(run_natives(vec![("not_str", not_str)], vec![Ld0U64, Call(p(FNS))]));
    match e {
        VmError::TypeError(errors) => match errors.as_slice() {
            [TypeError::NotEquals(t, _)] => assert_eq!(t.tag.to_string(), "arg0"),
            errors => panic!("{errors:?}"),
        },
        e => panic!("{e:?}"),
    }
}
