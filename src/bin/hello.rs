use ngvm::code::refs::*;
use ngvm::error::VmContextError;
use ngvm::model::Opcode::*;
use ngvm::{Code, ConstantPool, Vm};

fn main() -> Result<(), VmContextError> {
    let pool = ConstantPool::new(vec!["Hello world!".into()]);
    let code = Code::from_model(&[LdSS(p(0)), TraceStackValue(s(0))]).unwrap();
    let mut vm = Vm::light(pool);
    code.interpret(&mut vm)
}
