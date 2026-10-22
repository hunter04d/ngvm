use std::collections::HashMap;
use std::rc::Rc;

use smallvec::SmallVec;

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
mod pool;
pub mod primitives;
mod stack;
pub mod types;
pub mod vm;

pub struct Module {
    /// Blob of constants
    const_pool: ConstantPool,
    functions: HashMap<String, Rc<Function>>
}

impl Module {
    pub fn new(const_pool: ConstantPool) -> Self {
        Self { const_pool, functions: Default::default() }
    }

    pub fn add_fn(&mut self, s: String, f: Function) -> &mut Self {
        self.functions.insert(s, Rc::new(f));
        self
    }

    pub fn function(&self, name: &str) -> Option<&Rc<Function>> {
        self.functions.get(name)
    }
}

/// Function of a module, executed with the [`Call`](opcodes::Opcode::Call) opcode
///
/// The arguments are the top values of the caller's stack, they are moved into the function's
/// stack frame, so the function sees them as its first stack values.
/// The function must end with [`Ret`](opcodes::Opcode::Ret), or [`RetN`](opcodes::Opcode::RetN)
/// if it returns more than one value.
#[derive(Debug)]
pub struct Function {
    pub signature: Signature,
    pub bytecode: Code,
}

impl Function {
    pub fn new(signature: Signature, bytecode: Code) -> Self {
        Self { signature, bytecode }
    }
}

#[derive(Debug, PartialEq, Hash)]
pub struct Signature {
    params: Vec<VmType>,
    /// At least one, most functions return one value, which is stored inline
    returns: SmallVec<[VmType; 1]>,
}

impl Signature {
    /// Signature of a function that returns one value
    pub fn new(params: Vec<VmType>, return_type: impl Into<VmType>) -> Self {
        let returns = SmallVec::from_buf([return_type.into()]);
        Self { params, returns }
    }

    /// Signature of a function that returns multiple values, `None` if `returns` is empty
    pub fn with_returns(
        params: Vec<VmType>,
        returns: impl IntoIterator<Item = VmType>,
    ) -> Option<Self> {
        let returns: SmallVec<_> = returns.into_iter().collect();
        if returns.is_empty() {
            return None;
        }
        Some(Self { params, returns })
    }

    pub fn params(&self) -> &[VmType] {
        &self.params
    }

    pub fn returns(&self) -> &[VmType] {
        &self.returns
    }
}
