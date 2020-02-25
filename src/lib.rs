#![feature(allocator_api)]
#![feature(alloc_layout_extra)]

pub mod decoder;
pub mod interpreter;
pub mod model;
pub mod opcodes;
pub mod primitives;
pub mod types;

use std::collections::HashMap;

pub use code::Code;
pub use constant::{Constant, ConstantPool};
use stack_data::StackBytes;
pub use stack_value::StackValue;

mod code;
mod constant;
mod stack_data;
mod stack_value;

// TODO: enum or union
#[allow(dead_code)]
enum StackCell {
    StackValue(StackValue),
    StackBytes(StackBytes),
}

pub struct Vm {
    /// vm stack
    stack: Vec<StackValue>,
    /// current instruction in the current stack frame
    pub(crate) ip: usize,

    /// Index from which all indexing is happening
    pub(crate) last_stack_frame: usize,

    /// Index of the last pushed value
    pub(crate) last_pushed_value: usize,
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
            last_pushed_value: 0,
            modules: map,
            current_module: "".to_owned(),
        }
    }

    pub fn stack_value(&self, index: usize) -> &StackValue {
        &self.stack[self.last_stack_frame + index]
    }

    pub fn stack_value_mut(&mut self, index: usize) -> &mut StackValue {
        &mut self.stack[self.last_stack_frame + index]
    }

    pub fn push_single_stack_value(&mut self, v: StackValue) {
        self.last_pushed_value += 1;
        self.stack.push(v);
    }

    pub fn push_wide_stack_value(&mut self, v: StackValue, second: StackBytes) {
        self.last_pushed_value += 1;
        self.stack.push(v);
        // SAFETY: this is probably bad
        // TODO this is not the best way to do this, should probably use an enum or union
        self.stack.push(unsafe { std::mem::transmute(second) });
    }

    /// Pop the last stack value in its entirety from the stack
    pub fn pop_stack(&mut self) {
        // 1. [10, 11, 12]
        //             ^
        //             |
        // -------------
        // last_pushed_value = 3
        //
        // self.pop_stack()
        // 2. [10, 11]
        self.stack.truncate(self.last_pushed_value - 1)
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
