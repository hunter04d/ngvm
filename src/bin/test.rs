use ngvm::model::Opcode::*;
use ngvm::refs::{one, three, PoolRef};
use ngvm::types::PrimitiveType::*;
use ngvm::{Code, ConstantPool, Module, Vm};
use std::convert::TryInto;

fn main() {
    let pool = ConstantPool::new(vec![U64.into(), 1000u64.into(), 1u64.into()]);
    // spin up a vm instance
    println!("{:?}", pool);
    let mut vm = Vm::with_module(Module::new(pool));

    let code: Code = vec![
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
        TraceStackValue(one(0)),
        JC {
            label: 0,
            cond: one(6),
        },
    ]
    .try_into()
    .unwrap();
    let decode = code.decode();
    if !decode.is_full {
        panic!("Bad code")
    } else {
        decode.print(true);
    }

    code.interpret(&mut vm);
}
