use ngvm::code::refs::*;
use ngvm::model::{self, Opcode::*};

#[allow(dead_code)]
pub(super) fn fibonacci() -> Vec<model::Opcode> {
    vec![
        Ld0U64,
        Ld0U64,
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(1),
        },
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LdFalse,
        Ld0U64,
        Label(0),
        TraceStackValue(s(2)),
        UAdd(three(0, 1, 2)),
        Mv(s(1), s(2)),
        Mv(s(2), s(0)),
        UAdd(three(3, 3, 5)),
        Le(three(6, 3, 4)),
        JC {
            label: 0,
            cond: s(6),
        },
    ]
}

#[allow(dead_code)]
pub(super) fn fib_bytecode() -> Vec<u8> {
    vec![
        0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0,
        0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
        0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 6, 0, 254, 2, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0,
        0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 60, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0,
        0, 0, 0, 0, 60, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 3, 0, 0, 0, 0, 0, 0, 0,
        3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 42, 6, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0,
        0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 47, 72, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
    ]
}

#[allow(dead_code)]
pub(super) fn ref_test() -> Vec<model::Opcode> {
    vec![
        Ld0U64,
        Scope(vec![
            TakeRef(s(0)),
            Scope(vec![
                TakeRef(s(1)),
                Scope(vec![TakeMut(s(2)), TraceStackValue(s(3))]),
            ]),
        ]),
        Scope(vec![TakeMut(s(0)), TraceStackValue(s(1))]),
    ]
}

#[allow(dead_code)]
pub(super) fn test_deref() -> Vec<model::Opcode> {
    vec![
        Ld0U64,
        Scope(vec![
            TakeMut(s(0)),
            // 2
            LDType {
                type_location: p(0),
                value_location: p(1),
            },
            // 3
            StartDeref(s(1)),
            UAdd(three(3, 3, 2)),
            EndDeref,
        ]),
        TraceStackValue(s(0)),
    ]
}
