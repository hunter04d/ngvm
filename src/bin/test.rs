use ngvm::{Code, ConstantPool, Module, Vm};
use ngvm::model::Opcode::*;
use ngvm::types::Type::*;

fn main() {
    let pool = ConstantPool::new(vec![F64.into(), 10.0.into(), 20.0.into()]);
    // spin up a vm instance
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
        FAdd {
            result: 2,
            op1: 0,
            op2: 1,
        },
    ]
    .into();
    code.decode();
    code.interpret(&mut vm);
}
