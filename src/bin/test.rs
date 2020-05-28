use ngvm::error::{VmContextError, VmError};
use ngvm::model::{self, Opcode::*};
use ngvm::refs::{one, three, PoolRef};
use ngvm::types::PrimitiveType::*;
use ngvm::{Code, ConstantPool, Vm};

#[allow(dead_code)]
fn fibonacci() -> Vec<model::Opcode> {
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
        TraceStackValue(one(0)),
        JC {
            label: 0,
            cond: one(6),
        },
    ]
}

fn ref_test() -> Vec<model::Opcode> {
    vec! {
        Ld0U64,
        Scope(vec![
            TakeRef(one(0)),
            Scope(vec![
                TakeRef(one(1)),
                Scope(vec![
                    TakeMut(one(2)),
                    TraceStackValue(one(3)),
                ])
            ])
        ]),
        Scope(vec![
            TakeMut(one(0)),
            TraceStackValue(one(1))
        ])
    }
}

fn run(code: &[model::Opcode], pool: ConstantPool) -> Result<(), VmContextError> {
    // spin up a vm instance
    let mut vm = Vm::headless(pool);
    let code = Code::from_model(code).ok_or(VmError::InvalidBytecode)?;

    let decode = code.decode();
    if !decode.is_full {
        panic!("Bad code")
    } else {
        decode.print(true);
    }

    code.interpret(&mut vm)
}

fn main() {
    // let code = fibonacci();
    let code = ref_test();
    let pool = ConstantPool::new(vec![U64.into(), 10u64.into(), 1u64.into()]);
    let result = run(&code, pool);
    if let Err(e) = result {
        println!("{:?}", e);
    }
}
