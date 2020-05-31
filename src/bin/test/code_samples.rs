use ngvm::code::refs::*;
use ngvm::model::{self, Opcode::*};
use ngvm::types::PrimitiveType::*;
use ngvm::ConstantPool;

#[allow(dead_code)]
pub(super) fn fibonacci() -> Vec<model::Opcode> {
    vec![
        Ld0U64, // 0
        Ld0U64, // 1
        LDType {
            // 2
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LDType {
            // 3
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LDType {
            // 4 (100)
            type_location: PoolRef(0),
            value_location: PoolRef(1),
        },
        LDType {
            // 5
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LdFalse, // 6
        // TODO: MV instruction
        Ld0U64, // 7
        Label(0),
        // @0 = @1 + @2
        UAdd(three(0, 1, 2)),
        UAdd(three(1, 2, 7)),
        UAdd(three(2, 0, 7)),
        // @3 += '1';
        UAdd(three(3, 3, 5)),
        // if @3 <= @4
        Le(three(6, 3, 4)),
        TraceStackValue(s(0)),
        JC {
            label: 0,
            cond: s(6),
        },
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

#[allow(dead_code)]
pub(super) fn test_arr() -> (Vec<model::Opcode>, ConstantPool) {
    let max_size = 100_000 as usize;
    let mut code = Vec::with_capacity(max_size * 3);
    for i in 0..max_size {
        let l = &[StartScope, SArrCreate0(i, p(0)), EndScope];
        code.extend_from_slice(l);
    }
    let pool = ConstantPool::new(vec![U64.into()]);
    (code, pool)
}
