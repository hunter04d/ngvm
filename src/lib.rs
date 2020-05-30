#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(try_trait)]

use std::collections::HashMap;

pub use code::Code;
pub use pool::{Constant, ConstantPool};
pub use vm::Vm;

pub mod code;
pub mod decoder;
pub mod error;
pub mod interpreter;
pub mod meta;
pub mod model;
pub mod opcodes;
pub mod operations;
mod pool;
pub mod primitives;
mod stack;
pub mod types;
pub mod vm;

pub struct Module {
    /// Blob of constants
    const_pool: ConstantPool,
}

impl Module {
    pub fn new(const_pool: ConstantPool) -> Self {
        Self { const_pool }
    }
}

#[allow(dead_code)]
pub struct Function {
    signature: Signature,
    bytecode: Code,
}

#[derive(Eq, PartialEq, Hash)]
#[allow(dead_code)]
pub struct Signature {
    name: String,
    params: (),
    return_type: (),
}

#[allow(dead_code)]
pub struct ObjectDefinition {
    vtable: HashMap<Signature, Function>,
}
