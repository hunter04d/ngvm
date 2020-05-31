use ngvm::code::refs::*;
use ngvm::error::{VmContextError, VmError};
use ngvm::model;
use ngvm::model::Opcode::*;
use ngvm::types::PrimitiveType::*;
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
fn test_arr() -> (Vec<model::Opcode>, ConstantPool) {
    let max_size = 50_000 as usize;
    let mut code = Vec::with_capacity(max_size * 3);
    for i in 0..max_size {
        let l = &[StartScope, SArrCreate0(i, p(0)), EndScope];
        code.extend_from_slice(l);
    }
    let pool = ConstantPool::new(vec![U64.into()]);
    (code, pool)
}

fn main() {
    let (code, pool) = test_arr();
    let result = run(&code, pool, false);
    if let Err(e) = result {
        println!("{:?}", e);
    }
}
