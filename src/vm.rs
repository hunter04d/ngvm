use crate::error::VmError;
use crate::stack::{data::StackData, metadata::StackMetadata};
use crate::types::Type;
use crate::{ConstantPool, Module};
use std::collections::HashMap;
use crate::refs::{StackRef};

pub struct Vm {
    /// vm stack values
    pub(crate) stack: Vec<StackData>,
    /// vm stack metadata
    pub(crate) stack_metadata: Vec<StackMetadata>,
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
            stack_metadata: Vec::new(),
            ip: 0,
            last_stack_frame: 0,
            last_pushed_value: 0,
            modules: map,
            current_module: "".to_owned(),
        }
    }

    pub fn stack_data(&self, index: StackRef) -> Result<&StackData, VmError> {
        self.stack
            .get(self.last_stack_frame + index.0)
            .ok_or(VmError::BadVmState)
    }
    pub fn stack_data_opt(&self, index: StackRef) -> Option<&StackData> {
        self.stack.get(self.last_stack_frame + index.0)
    }

    pub fn stack_metadata(&self, index: StackRef) -> Result<&StackMetadata, VmError> {
        self.stack_metadata
            .get(self.last_stack_frame + index.0)
            .ok_or(VmError::BadVmState)
    }

    pub fn stack_data_mut(&mut self, index: StackRef) -> Result<&mut StackData, VmError> {
        self.stack
            .get_mut(self.last_stack_frame + index.0)
            .ok_or(VmError::BadVmState)
    }

    pub fn push_stack_data_with_type(&mut self, value: StackData, t: Type) {
        self.last_pushed_value += 1;
        self.stack_metadata
            .push(StackMetadata::new(t, StackRef(self.stack.len())));
        self.stack.push(value);
    }

    pub fn push_default_with_type(&mut self, t: Type) {
        self.last_pushed_value += 1;
        self.stack_metadata
            .push(StackMetadata::new(t, StackRef(self.stack.len())));
        self.stack.push(Default::default());
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
        self.stack.truncate(self.last_pushed_value - 1);
        self.stack_metadata.truncate(self.last_pushed_value - 1);
    }

    pub fn current_const_pool(&self) -> &ConstantPool {
        &self.modules[&self.current_module].const_pool
    }
}
