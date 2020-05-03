use ngvm::model::Opcode::*;
use ngvm::refs::{three, PoolRef, StackRef, one};
use ngvm::types::Type::*;
use ngvm::{Code, ConstantPool, Module, Vm};

fn main() {
    let pool = ConstantPool::new(vec![F32.into(), 10.0_f32.into(), 20.0_f32.into()]);
    // spin up a vm instance
    println!("{:?}", pool);
    let mut vm = Vm::with_module(Module::new(pool));
    let code: Code = vec![
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(1),
        },
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LDTyped0 { type_location: PoolRef(0) },
        FAdd(three(2, 0, 1)),
        TraceStackValue(one(2)),
    ]
    .into();
    let decode = code.decode();
    if !decode.is_full {
        panic!("Bad code")
    } else {
        decode.print(true);
    }

    code.interpret(&mut vm);
}
