#![feature(allocator_api)]
#![feature(alloc_layout_extra)]

use std::collections::HashMap;

pub use code::Code;
pub use constant::{Constant, ConstantPool};
pub use vm::Vm;

pub mod decoder;
pub mod interpreter;
pub mod model;
pub mod opcodes;
pub mod primitives;
pub mod types;
pub mod refs;

mod code;
mod constant;
mod stack;
mod vm;

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
