use ngvm::code::refs::*;
use ngvm::model::{self, Opcode::*};
use std::fs::File;
use std::io;

fn fibonacci() -> Vec<model::Opcode> {
    vec![
        Ld0U64,
        Ld0U64,
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(1),
        },
        LDType {
            type_location: PoolRef(0),
            value_location: PoolRef(2),
        },
        LdFalse,
        Ld0U64,
        Label(0),
        TraceStackValue(s(2)),
        UAdd(three(0, 1, 2)),
        Mv(s(1), s(2)),
        Mv(s(2), s(0)),
        UAdd(three(3, 3, 5)),
        Le(three(6, 3, 4)),
        JC {
            label: 0,
            cond: s(6),
        },
    ]
}

fn main() -> io::Result<()> {
    let m = fibonacci();
    let m = m.as_slice();
    let file = File::create("data/fib.yaml")?;
    serde_yaml::to_writer(file, m).unwrap();
    let file = File::create("data/fib.bincode")?;
    bincode::serialize_into(file, m).unwrap();
    let file = File::create("data/fib.json")?;
    serde_json::to_writer(file, m).unwrap();
    Ok(())
}
