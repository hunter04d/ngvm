use std::collections::HashMap;

use crate::code::{Chunk, RefSource};
use crate::error::VmError;
use crate::refs::{PoolRef, Ref, StackRef, ThreeStackRefs, TwoStackRefs};
use crate::stack::{data::StackData, metadata::StackMetadata};
use crate::types::PrimitiveType;
use crate::vm::lock::ValueLock;
use crate::{ConstantPool, Module};

pub mod lock;

pub struct Vm {
    /// vm stack values
    pub(crate) stack: Vec<StackData>,
    /// vm stack metadata
    pub(crate) stack_metadata: Vec<StackMetadata>,
    /// current instruction in the current stack frame
    pub(crate) ip: usize,

    /// Index from which all indexing is happening
    pub(crate) last_stack_frame: usize,

    /// The current lifecycle of the vm, starts at one
    ///
    /// Zero is reserved for static data segment in the future
    pub(crate) cycle: usize,
    /// Loaded modules
    pub(crate) modules: HashMap<String, Module>,

    pub(crate) current_module: String,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct StackDataRef(pub usize);

impl Vm {
    pub fn with_module(m: Module) -> Self {
        let mut map = HashMap::new();
        map.insert("".to_string(), m);
        Self {
            modules: map,
            ..Default::default()
        }
    }

    pub fn headless(pool: ConstantPool) -> Self {
        let module = Module::new(pool);
        Self::with_module(module)
    }

    pub fn default_growing_stack() -> Self {
        Self {
            stack: Vec::new(),
            stack_metadata: Vec::new(),
            ..Default::default()
        }
    }

    pub fn single_stack_data(&self, index: StackRef) -> Result<&StackData, VmError> {
        let meta = self.stack_metadata(index)?;
        if meta.value_type.size() == 1 {
            self.stack
                .get(self.last_stack_frame + meta.index.0)
                .ok_or(VmError::BadVmState)
        } else {
            Err(VmError::BadVmState)
        }
    }

    pub fn stack_data(&self, meta: &StackMetadata) -> Result<&[StackData], VmError> {
        let index = meta.index.0;
        let size = meta.value_type.size();
        let from = self.last_stack_frame + index;
        let until = from + size;
        if until > self.stack.len() {
            Err(VmError::BadVmState)
        } else {
            Ok(&self.stack[from..until])
        }
    }
    pub fn stack_data_opt(&self, index: StackDataRef) -> Option<&StackData> {
        self.stack.get(self.last_stack_frame + index.0)
    }

    pub fn stack_metadata(&self, index: StackRef) -> Result<&StackMetadata, VmError> {
        self.stack_metadata
            .get(self.last_stack_frame + index.0)
            .ok_or(VmError::BadVmState)
    }

    pub fn single_stack_data_mut(&mut self, index: StackRef) -> Result<&mut StackData, VmError> {
        let meta = self
            .stack_metadata
            .get(self.last_stack_frame + index.0)
            .ok_or(VmError::BadVmState)?;
        if meta.value_type.size() == 1 {
            self.stack
                .get_mut(self.last_stack_frame + meta.index.0)
                .ok_or(VmError::BadVmState)
        } else {
            Err(VmError::BadVmState)
        }
    }

    pub fn push_primitive(&mut self, value: StackData, t: PrimitiveType) {
        let len = self.stack.len();
        self.stack_metadata.push(StackMetadata {
            value_type: t.into(),
            index: StackDataRef(len),
            cycle: self.cycle,
            lock: ValueLock::None,
        });
        self.stack.push(value);
    }

    pub fn push_primitive_zeroed(&mut self, t: PrimitiveType) {
        self.push_primitive(Default::default(), t)
    }

    /// Pop the last stack value in its entirety from the stack
    pub fn pop_stack(&mut self) {
        if let Some(meta) = self.stack_metadata.last() {
            let size = meta.value_type.size();
            self.stack.truncate(self.stack.len() - size);
        }
    }

    pub fn current_const_pool(&self) -> &ConstantPool {
        &self.modules[&self.current_module].const_pool
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self {
            stack: Vec::with_capacity(128),
            stack_metadata: Vec::with_capacity(128),
            ip: 0,
            last_stack_frame: 0,
            cycle: 1,
            modules: Default::default(),
            current_module: "".to_string(),
        }
    }
}

pub trait VmRefSource {
    type VmError: std::error::Error;

    fn read_from_offset_vm(&self, offset: usize, size: usize) -> Result<&[u8], Self::VmError>;

    fn read_ref_vm(&self, index: usize) -> Result<Ref, Self::VmError>;

    fn read_ref_with_offset_vm(&self, index: usize) -> Result<Ref, Self::VmError>;

    fn read_offset_vm(&self) -> Result<usize, Self::VmError>;

    fn read_two_vm(&self) -> Result<TwoStackRefs, Self::VmError> {
        let result = StackRef(self.read_ref_vm(0)?);
        let op = StackRef(self.read_ref_vm(1)?);
        Ok(TwoStackRefs { result, op })
    }

    fn read_three_vm(&self) -> Result<ThreeStackRefs, Self::VmError> {
        let result = StackRef(self.read_ref_vm(0)?);
        let op1 = StackRef(self.read_ref_vm(1)?);
        let op2 = StackRef(self.read_ref_vm(2)?);
        Ok(ThreeStackRefs { result, op1, op2 })
    }

    fn read_ref_pool_vm(&self, index: usize) -> Result<PoolRef, Self::VmError> {
        Ok(PoolRef(self.read_ref_vm(index)?))
    }

    fn read_ref_stack_vm(&self, index: usize) -> Result<StackRef, Self::VmError> {
        Ok(StackRef(self.read_ref_vm(index)?))
    }

    fn read_ref_stack_with_offset_vm(&self, index: usize) -> Result<StackRef, Self::VmError> {
        self.read_ref_with_offset_vm(index).map(StackRef)
    }
}

impl VmRefSource for Chunk<'_> {
    type VmError = VmError;

    fn read_from_offset_vm(&self, offset: usize, size: usize) -> Result<&[u8], Self::VmError> {
        self.read_from_offset(offset, size)
            .ok_or(VmError::InvalidBytecode)
    }

    fn read_ref_vm(&self, index: usize) -> Result<Ref, Self::VmError> {
        self.read_ref(index).ok_or(VmError::InvalidBytecode)
    }

    fn read_ref_with_offset_vm(&self, index: usize) -> Result<Ref, Self::VmError> {
        self.read_ref_with_offset(index)
            .ok_or(VmError::InvalidBytecode)
    }

    fn read_offset_vm(&self) -> Result<usize, Self::VmError> {
        self.read_offset().ok_or(VmError::InvalidBytecode)
    }
}
