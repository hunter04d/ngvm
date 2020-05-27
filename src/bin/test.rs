use std::fs::File;

use flate2::Compression;
use flate2::write::GzEncoder;

use ngvm::{Code, ConstantPool, Vm};
use ngvm::error::{VmContextError, VmError};
use ngvm::model::{self, Opcode::*};
use ngvm::refs::{one, PoolRef, three};
use ngvm::types::PrimitiveType::*;
use serde::Serialize;

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
    let code = fibonacci();
    let pool = ConstantPool::new(vec![U64.into(), 10u64.into(), 1u64.into()]);
    let result = run(&code, pool);
    if let Err(e) = result {
        println!("{:?}", e);
    }

    let file = File::create("data/test.yaml").unwrap();
    serde_yaml::to_writer(file, &code).unwrap();

    let file = File::create("data/test.ngvm").unwrap();

    let df = GzEncoder::new(file, Compression::best());
    // why are you like so?
    bincode::serialize_into(df, &code).unwrap();
    let file = File::create("data/test.json").unwrap();
    serde_json::to_writer_pretty(file, &code).unwrap();

    let config = ron::ser::PrettyConfig::default().with_extensions(ron::extensions::Extensions::UNWRAP_NEWTYPES);
    let file = File::create("data/test.ron").unwrap();
    let mut ser = ron::Serializer::new(file, Some(config), false).unwrap();
    code.serialize(&mut ser).unwrap();
}
