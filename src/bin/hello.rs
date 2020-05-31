use ngvm::code::refs::*;
use ngvm::error::{VmContextError, VmError};
use ngvm::model;
use ngvm::model::Opcode::*;
use ngvm::{Code, ConstantPool, Vm};

fn run(code: &[model::Opcode], pool: ConstantPool, do_decode: bool) -> Result<(), VmContextError> {
    // spin up a vm instance
    let mut vm = Vm::headless(pool);
    let code = Code::from_model(code).ok_or(VmError::InvalidBytecode)?;

    if do_decode {
        let decode = code.decode();
        if !decode.is_full {
            panic!("Bad code")
        } else {
            decode.print(true);
        }
    }

    code.interpret(&mut vm)
}

#[allow(dead_code)]
fn hello() -> (Vec<model::Opcode>, ConstantPool) {
    let code = vec![LdSS(p(0)), TraceStackValue(s(0))];
    let pool = ConstantPool::new(vec!["Hello world!".into()]);
    (code, pool)
}

fn main() {
    let (code, pool) = hello();
    let result = run(&code, pool, false);
    if let Err(e) = result {
        println!("{:?}", e);
    }
}
