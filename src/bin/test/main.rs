use code_samples::*;
use ngvm::code::refs::{p, s, three};
use ngvm::error::VmContextError;
use ngvm::model::Opcode::*;
use ngvm::types::PrimitiveType::*;
use ngvm::{Code, ConstantPool, Vm};

mod code_samples;

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
    let model = vec![
        // 0
        SArrCreate0(10, p(0)),
        StartScope,
        // 1
        TakeMut(s(0)),
        // 2
        Ld0U64,
        // 3
        StartScope,
        SArrMut {
            arr_mut: s(1),
            index: s(2),
        },
        LDType {
            type_location: p(0),
            value_location: p(1),
        },
        // 5
        StartDeref(s(3)),
        UAdd(three(5, 5, 4)),
        EndDeref,
        EndScope,
        EndScope,
        TraceStackValue(s(0)),
    ];
    let code = Code::from_model(&model).unwrap();
    let result = run(&code, pool, false);
    if let Err(e) = result {
        println!("{:#?}", e);
    }
}
