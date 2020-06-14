#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(try_trait)]

use std::collections::HashMap;

pub use code::Code;
pub use pool::{Constant, ConstantPool};
pub use vm::Vm;
use crate::types::VmType;

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
    functions: HashMap<String, Function>
}

impl Module {
    pub fn new(const_pool: ConstantPool) -> Self {
        Self { const_pool, functions: Default::default() }
    }

    pub fn add_fn(&mut self, s: String, f: Function) -> &mut Self {
        self.functions.insert(s, f);
        self
    }
}

pub struct Function {
    pub signature: Signature,
    pub bytecode: Code,
}

#[derive(PartialEq, Hash)]
pub struct Signature {
    params: Vec<VmType>,
    return_type: VmType,
}
