use ngvm::error::{VmContextError, VmError};
use ngvm::model;
use ngvm::types::PrimitiveType::*;
use ngvm::{Code, ConstantPool, Vm};

mod code_samples;
use code_samples::*;

fn default_pool() -> ConstantPool {
    ConstantPool::new(vec![U64.into(), 10u64.into(), 1u64.into()])
}

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

fn main() {
    let _ = default_pool();
    let (code, pool) = test_arr();
    let result = run(&code, pool, false);
    if let Err(e) = result {
        println!("{:?}", e);
    }
}
