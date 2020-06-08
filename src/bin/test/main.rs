use ngvm::error::VmContextError;
use ngvm::types::PrimitiveType::*;
use ngvm::{Code, ConstantPool, Vm};

mod code_samples;
use code_samples::*;

fn default_pool() -> ConstantPool {
    ConstantPool::new(vec![U64.into(), 90u64.into(), 1u64.into()])
}

fn run(code: &Code, pool: ConstantPool, do_decode: bool) -> Result<(), VmContextError> {
    // spin up a vm instance
    if do_decode {
        let decode = code.decode();
        if !decode.is_full {
            panic!("Bad code")
        } else {
            decode.print(true);
        }
    }
    let mut vm = Vm::headless(pool);
    code.interpret(&mut vm)
}

fn main() {
    let pool = default_pool();
    let code = Code::from_model(&fibonacci()).unwrap();
    let result = run(&code, pool, false);
    if let Err(e) = result {
        println!("{:?}", e);
    }
}
