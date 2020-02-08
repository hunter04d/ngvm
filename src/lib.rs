#![feature(allocator_api)]
#![feature(alloc_layout_extra)]

pub mod decoder;
pub mod interpreter;
pub mod model;
pub mod opcodes;
pub mod primitives;
pub mod types;

use std::collections::HashMap;
use std::convert::TryInto;
use std::mem::size_of;

pub use code::Code;
pub use constant::{Constant, ConstantPool};
pub use stack_value::StackValue;

mod code;
mod constant;
mod stack_value;

pub struct Vm {
    /// vm stack
    pub(crate) stack: Vec<StackValue>,
    /// current instruction in the current stack frame
    pub(crate) ip: usize,

    /// Index from which all indexing is happening
    pub(crate) last_stack_frame: usize,
    /// Loaded modules
    pub(crate) modules: HashMap<String, Module>,

    pub(crate) current_module: String,
}

impl Vm {
    pub fn with_module(m: Module) -> Self {
        let mut map = HashMap::new();
        map.insert("".to_string(), m);
        Self {
            stack: Vec::new(),
            ip: 0,
            last_stack_frame: 0,
            modules: map,
            current_module: "".to_owned(),
        }
    }

    pub fn current_const_pool(&self) -> &ConstantPool {
        &self.modules[&self.current_module].const_pool
    }
}

pub struct Module {
    /// Blob of constants
    const_pool: ConstantPool,
}

impl Module {
    pub fn new(const_pool: ConstantPool) -> Self {
        Self { const_pool }
    }
}

pub struct Function {
    signature: Signature,
    bytecode: Code,
}

#[derive(Eq, PartialEq, Hash)]
pub struct Signature {
    name: String,
    params: (),
    return_type: (),
}

pub struct ObjectDefinition {
    vtable: HashMap<Signature, Function>,
}
