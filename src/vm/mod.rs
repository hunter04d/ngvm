use std::collections::HashMap;

use crate::code::refs::StackRef;
use crate::error::VmError;
use crate::meta::{Meta, StackMeta, TransientMeta};
use crate::stack::data::IntoStackData;
use crate::stack::data::StackData;
use crate::types::{PointedType, PrimitiveType, RefKind, RefLocation, RefType, VmType};
use crate::{ConstantPool, Module};
use lock::ValueLock;
use refs::LocatedRef;

pub mod lock;
pub mod refs;

pub use refs::code::VmRefSource;

pub struct Vm {
    /// vm stack values
    pub(crate) stack: Vec<StackData>,
    /// vm stack metadata
    pub(crate) stack_metadata: Vec<StackMeta>,

    pub(crate) transient_refs: HashMap<ValueLocation, TransientMeta>,

    pub(crate) derefs: Vec<VmDeref>,
    /// The current cycle of the vm
    ///
    /// Starts at 1. 0 is reserved for static data
    pub(crate) cycle: usize,
    /// current instruction in the current stack frame
    pub(crate) ip: usize,

    /// Index from which all indexing is happening
    pub(crate) last_stack_frame: usize,

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

    pub fn light(pool: ConstantPool) -> Self {
        let module = Module::new(pool);
        let mut map = HashMap::new();
        map.insert("".into(), module);
        Self {
            stack: Vec::new(),
            stack_metadata: Vec::new(),
            transient_refs: HashMap::new(),
            derefs: Vec::new(),
            cycle: 1,
            ip: 0,
            last_stack_frame: 0,
            modules: map,
            current_module: "".into(),
        }
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

    pub fn stack_data(&self, index: StackRef) -> Result<&[StackData], VmError> {
        let meta = self.stack_metadata(index)?;
        let from = meta.index.0;
        let until = from + meta.value_type.size();
        self.stack.get(from..until).ok_or(VmError::BadVmState)
    }

    pub fn stack_metadata(&self, index: StackRef) -> Result<&StackMeta, VmError> {
        self.stack_metadata
            .get(self.last_stack_frame + index.0)
            .ok_or(VmError::BadVmState)
    }

    pub fn stack_metadata_mut(&mut self, index: StackRef) -> Result<&mut StackMeta, VmError> {
        self.stack_metadata
            .get_mut(self.last_stack_frame + index.0)
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

    pub fn stack_data_mut(&mut self, index: StackRef) -> Result<&mut [StackData], VmError> {
        let meta = self.stack_metadata(index)?;
        let from = meta.index.0;
        let until = from + meta.value_type.size();
        Ok(&mut self.stack[from..until])
    }

    pub fn push_primitive(&mut self, value: StackData, t: PrimitiveType) {
        let len = self.stack.len();
        let cycle = self.current_cycle();
        let meta = StackMeta::new(t, StackDataRef(len), cycle);

        self.stack_metadata.push(meta);
        self.stack.push(value);
    }

    pub fn push_primitive_zeroed(&mut self, t: PrimitiveType) {
        self.push_primitive(Default::default(), t)
    }

    pub fn push_stack_ref(&mut self, index: StackRef, kind: RefKind) -> Result<(), VmError> {
        let cycle = self.current_cycle();
        let meta = self.stack_metadata_mut(index)?;
        let lock = &mut meta.lock;
        match lock.add_lock(cycle, kind) {
            Ok(()) => {
                let pointer = meta.value_type.clone();
                let len = self.stack.len();
                let ref_type = PointedType::reference(pointer, kind, RefLocation::Stack);
                let ref_meta = StackMeta::new(ref_type, StackDataRef(len), cycle);
                self.stack_metadata.push(ref_meta);
                self.stack.push(index.0.into_stack_data());
                Ok(())
            }
            Err(e) => Err(VmError::LockError(e, index)),
        }
    }

    /// Pop the last stack value in its entirety from the stack
    pub fn pop_stack(&mut self) -> Result<(), VmError> {
        if let Some(meta) = self.stack_metadata.pop() {
            let size = meta.value_type.size();
            match meta.value_type {
                VmType::Primitive(_) => {}
                VmType::PointedType(p) => match *p {
                    PointedType::SArr(_) => {}
                    PointedType::Ref(r) => {
                        let ref_value = self
                            .stack
                            .get(self.last_stack_frame + meta.index.0)
                            .ok_or(VmError::BadVmState)?;
                        let located_ref = r.locate(ref_value);
                        self.unlock_by_ref(located_ref)?;
                    }
                },
            }
            self.stack.truncate(self.stack.len() - size);
        }
        Ok(())
    }

    pub fn free_by_index(&mut self, index: StackRef) -> Result<(), VmError> {
        let meta = self.stack_metadata(index)?;
        let is_copy = meta.value_type.is_copy();
        match &meta.value_type {
            VmType::Primitive(_) => {}
            VmType::PointedType(p) => match p.as_ref() {
                PointedType::SArr(_) => {}
                PointedType::Ref(r) => {
                    let ref_value = self
                        .stack
                        .get(self.last_stack_frame + meta.index.0)
                        .ok_or(VmError::BadVmState)?;
                    let located_ref = r.locate(ref_value);
                    self.unlock_by_ref(located_ref)?;
                }
            },
        }
        if !is_copy {
            self.stack_metadata_mut(index)?.was_moved = true;
        }
        Ok(())
    }

    fn unlock_by_ref(&mut self, rf: LocatedRef) -> Result<(), VmError> {
        let vm_cycle = self.cycle;
        match rf {
            LocatedRef::Stack(index) => {
                let value_meta = self.stack_metadata_mut(StackRef(index))?;
                if let Some(c) = value_meta.lock.lock_cycle() {
                    if c == vm_cycle {
                        value_meta.lock = ValueLock::None;
                    }
                }
            }
            LocatedRef::Transient(index) => {
                let value_meta = self
                    .transient_refs
                    .get_mut(&index)
                    .ok_or(VmError::BadVmState)?;
                if let Some(c) = value_meta.lock.lock_cycle() {
                    if c == vm_cycle {
                        value_meta.lock = ValueLock::None;
                        let root = value_meta.root_object;
                        self.unlock_by_ref(root)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn switch_lock_cycle(&mut self, rf: LocatedRef) -> Result<(), VmError> {
        fn switch_cycle(m: &mut impl Meta, new_cycle: usize) -> Result<(), VmError> {
            match m.lock() {
                ValueLock::Mut(current_cycle) if new_cycle >= *current_cycle => {
                    *m.lock_mut() = ValueLock::Mut(new_cycle);
                    Ok(())
                }
                // TODO: different error
                ValueLock::Mut(_) => Err(VmError::BadVmState),
                _ => Err(VmError::BadVmState),
            }
        }
        let vm_cycle = self.cycle;
        match rf {
            LocatedRef::Stack(index) => {
                let value_meta = self.stack_metadata_mut(StackRef(index))?;
                switch_cycle(value_meta, vm_cycle)
            }
            LocatedRef::Transient(index) => {
                let value_meta = self
                    .transient_refs
                    .get_mut(&index)
                    .ok_or(VmError::BadVmState)?;
                switch_cycle(value_meta, vm_cycle)
            }
        }
    }

    pub fn push_scope(&mut self) -> Result<(), VmError> {
        self.cycle = self.cycle.checked_add(1).ok_or(VmError::BadVmState)?;
        Ok(())
    }

    pub fn pop_scope(&mut self) -> Result<(), VmError> {
        if self.cycle == 1 {
            Err(VmError::BadVmState)
        } else {
            self.cycle -= 1;
            Ok(())
        }
    }

    pub fn stack_meta_of_type(&self, t: VmType) -> StackMeta {
        let cycle = self.current_cycle();
        let len = self.stack.len();
        StackMeta::new(t, StackDataRef(len), cycle)
    }

    pub fn push_deref(&mut self, value: &[StackData], t: VmType, kind: RefKind, rf: StackRef) {
        let len = self.stack_metadata.len();
        let mut meta = self.stack_meta_of_type(t);
        meta.deref = kind.into();
        self.stack_metadata.push(meta);
        self.stack.extend_from_slice(value);
        self.derefs.push(VmDeref {
            rf,
            deref: StackRef(len),
        });
    }

    pub fn pop_deref(&mut self) -> Result<(), VmError> {
        if let Some(d) = self.derefs.pop() {
            let (lr, rt) = self.locate_ref(d.rf)?;
            let pointer_size = rt.pointer.size();
            if rt.kind == RefKind::Mut {
                self.stack_metadata_mut(d.rf)?.lock = ValueLock::None;
                let deref_data = self.stack_data(d.deref)?.to_vec();
                match lr {
                    LocatedRef::Stack(index) => {
                        let from = index;
                        let until = from + pointer_size;
                        self.stack.splice(from..until, deref_data);
                    }
                    LocatedRef::Transient(index) => match index {
                        ValueLocation::Stack(index) => {
                            let from = index;
                            let until = from + pointer_size;
                            self.stack.splice(from..until, deref_data);
                        }
                        ValueLocation::Heap(_) => unimplemented!(),
                    },
                }
                self.stack_metadata_mut(d.deref)?.was_moved = true;
            }

            Ok(())
        } else {
            Err(VmError::BadVmState)
        }
    }

    pub fn locate_ref(&self, index: StackRef) -> Result<(LocatedRef, &RefType), VmError> {
        let meta = self.stack_metadata(index)?;
        if let Some(PointedType::Ref(r)) = meta.value_type.pointed() {
            let data = self.single_stack_data(index)?;
            Ok((r.locate(data), r))
        } else {
            Err(VmError::BadVmState)
        }
    }

    pub fn push_array_0(&mut self, size: usize, t: PrimitiveType) {
        let arr_type = PointedType::s_arr(t, size);
        let stack_size = arr_type.size();
        let meta = self.stack_meta_of_type(arr_type.into());
        self.stack_metadata.push(meta);
        self.stack
            .extend(std::iter::repeat(StackData::default()).take(stack_size))
    }

    pub fn push_s_str(&mut self, ptr: usize, len: usize) {
        let meta = self.stack_meta_of_type(PrimitiveType::SStr.into());
        self.stack_metadata.push(meta);
        self.stack.push(ptr.into_stack_data());
        self.stack.push(len.into_stack_data());
    }

    pub fn current_cycle(&self) -> usize {
        self.cycle
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
            cycle: 1,
            ip: 0,
            last_stack_frame: 0,
            modules: Default::default(),
            current_module: "".to_string(),
            transient_refs: HashMap::new(),
            derefs: Vec::new(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum ValueLocation {
    Stack(usize),
    Heap(*const ()),
}

#[derive(Debug)]
pub struct VmDeref {
    pub rf: StackRef,
    pub deref: StackRef,
}
