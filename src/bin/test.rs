use ngvm::model::refs::three;
use ngvm::model::Opcode::*;
use ngvm::types::Type::*;
use ngvm::{Code, ConstantPool, Module, Vm};

fn main() {
    let pool = ConstantPool::new(vec![F32.into(), 10.0_f32.into(), 20.0_f32.into()]);
    // spin up a vm instance
    println!("{:?}", pool);
    let mut vm = Vm::with_module(Module::new(pool));
    let code: Code = vec![
        LDType {
            type_location: 0,
            value_location: 1,
        },
        LDType {
            type_location: 0,
            value_location: 2,
        },
        LDTyped0 { type_location: 0 },
        FAdd(three(2, 0, 1)),
        TraceStackValue(2),
        FRem(three(1, 1, 0)),
        TraceStackValue(1),
    ]
    .into();
    code.decode();
    code.interpret(&mut vm);
}
