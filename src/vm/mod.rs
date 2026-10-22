use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use lock::ValueLock;
pub use refs::code::VmRefSource;
use refs::LocatedRef;
use smallvec::SmallVec;

use crate::{ConstantPool, Function, FunctionBody, Module};
use crate::code::refs::{PoolRef, StackRef};
use crate::error::VmError;
use crate::meta::{StackMeta, TransientMeta, VmMetaView};
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::stack::get_stack_range;
use crate::types::{PointedType, PrimitiveType, RefKind, RefType, VmType};
use crate::types::checker::{Taggable, TypeError};
use crate::vm::lock::DerefLock;

pub mod lock;
pub mod native;
pub mod refs;
#[cfg(test)]
mod tests;

/// # Memory model
///
/// Only bytecode can change the state of the vm, the methods that do are crate private
/// so that the checks below cannot be bypassed. Native functions can only access their own
/// stack frame through [`NativeCall`](native::NativeCall), which uses the same methods.
///
/// Values live on the stack and are only ever removed from its top, so a value at a lower
/// index always outlives a value at a higher one.
///
/// Every live reference holds one borrow (see [`ValueLock`]) on the value it points to and
/// releases it when it stops existing (popped or overwritten), unless it was moved out.
/// A reference may only be stored above the value it points to, so it can never outlive it.
///
/// A function call moves its arguments into a new stack frame, which only the callee can access.
/// Returning pops the whole frame and moves the returned value onto the caller's stack,
/// so a reference into the frame cannot be returned.
///
/// A static string value points to a string of a constant pool (see [`static_str`]).
/// Modules are never removed or replaced, so the string lives as long as the vm.
pub struct Vm {
    /// vm stack values
    pub(crate) stack: Vec<StackData>,
    /// vm stack metadata
    pub(crate) stack_metadata: Vec<StackMeta>,

    /// Borrowed values that are not stack values of their own (e.g. array elements)
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

    /// Loaded modules, they are never removed or replaced
    pub(crate) modules: HashMap<String, Module>,

    pub(crate) current_module: String,

    /// Active function calls, the last one is executing
    pub(crate) frames: Vec<CallFrame>,
}

pub type Result<T> = std::result::Result<T, VmError>;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct StackDataRef(pub usize);

/// How an instruction is going to use a stack value
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Access {
    /// Read the value and leave it in place
    Read,
    /// Overwrite the whole value
    Write,
    /// Move the value out of its slot, a `Copy` value is only read
    Move,
}

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
            frames: Vec::new(),
        }
    }

    /// Loads the module `m` as `name`, fails if a module with that name is already loaded
    pub fn add_module(&mut self, name: impl Into<String>, m: Module) -> Result<&mut Self> {
        match self.modules.entry(name.into()) {
            Entry::Occupied(e) => Err(VmError::ModuleAlreadyLoaded(e.key().clone())),
            Entry::Vacant(e) => {
                e.insert(m);
                Ok(self)
            }
        }
    }

    pub fn default_growing_stack() -> Self {
        Self {
            stack: Vec::new(),
            stack_metadata: Vec::new(),
            ..Default::default()
        }
    }

    /// Absolute index of the stack value, i.e. not relative to the current stack frame
    pub fn absolute_index(&self, index: StackRef) -> usize {
        self.last_stack_frame + index.0
    }

    pub fn single_stack_data(&self, index: StackRef) -> Result<&StackData> {
        let meta = self.stack_metadata(index)?;
        if meta.value_type.size() == 1 {
            self.stack.get(meta.index.0).ok_or(VmError::BadVmState)
        } else {
            Err(VmError::BadVmState)
        }
    }

    pub fn stack_data(&self, index: StackRef) -> Result<&[StackData]> {
        let meta = self.stack_metadata(index)?;
        self.stack
            .get(get_stack_range(meta.index.0, meta))
            .ok_or(VmError::BadVmState)
    }

    pub fn stack_metadata(&self, index: StackRef) -> Result<&StackMeta> {
        self.absolute_metadata(self.absolute_index(index))
    }

    pub fn absolute_metadata(&self, index: usize) -> Result<&StackMeta> {
        self.stack_metadata.get(index).ok_or(VmError::BadVmState)
    }

    /// Gets the metadata of a stack value after checking that it can be used for `access`
    pub fn checked_stack_metadata(&self, index: StackRef, access: Access) -> Result<&StackMeta> {
        let meta = self.stack_metadata(index)?;
        let lock_error = |e| VmError::LockError(e, ValueLocation::Stack(index.0));
        let access = if access == Access::Move && meta.value_type.is_copy() {
            Access::Read
        } else {
            access
        };
        match access {
            Access::Read | Access::Move if meta.was_moved => {
                return Err(VmError::UseOfMovedValue(index));
            }
            Access::Read => meta.lock.check_read().map_err(lock_error)?,
            Access::Write | Access::Move => meta.lock.check_write().map_err(lock_error)?,
        }
        Ok(meta)
    }

    pub fn meta_view(&self, l: LocatedRef) -> Result<VmMetaView<'_>> {
        match l {
            LocatedRef::Stack(index) => self.absolute_metadata(index).map(VmMetaView::Stack),
            LocatedRef::Transient(tr) => self
                .transient_refs
                .get(&tr)
                .map(VmMetaView::Transient)
                .ok_or(VmError::BadVmState),
        }
    }

    pub(crate) fn stack_metadata_mut(&mut self, index: StackRef) -> Result<&mut StackMeta> {
        self.absolute_metadata_mut(self.absolute_index(index))
    }

    pub(crate) fn absolute_metadata_mut(&mut self, index: usize) -> Result<&mut StackMeta> {
        self.stack_metadata.get_mut(index).ok_or(VmError::BadVmState)
    }

    pub(crate) fn single_stack_data_mut(&mut self, index: StackRef) -> Result<&mut StackData> {
        let meta = self.stack_metadata(index)?;
        if meta.value_type.size() == 1 {
            let data_index = meta.index.0;
            self.stack.get_mut(data_index).ok_or(VmError::BadVmState)
        } else {
            Err(VmError::BadVmState)
        }
    }

    /// Overwrites a single stack value, which (re)initializes it
    pub(crate) fn set_single_stack_data(&mut self, index: StackRef, value: StackData) -> Result<()> {
        *self.single_stack_data_mut(index)? = value;
        self.stack_metadata_mut(index)?.was_moved = false;
        Ok(())
    }

    pub(crate) fn push_single_typed(&mut self, value: impl IntoStackData, t: impl Into<VmType>) {
        let len = self.stack.len();
        let cycle = self.current_cycle();
        let meta = StackMeta::new(t, StackDataRef(len), cycle);

        self.stack_metadata.push(meta);
        self.stack.push(value.into_stack_data());
    }

    pub(crate) fn push_primitive_zeroed(&mut self, t: PrimitiveType) {
        self.push_single_typed(StackData::default(), t)
    }

    /// Borrows the stack value at `index` and pushes a reference of `kind` to it
    pub(crate) fn push_stack_ref(&mut self, index: StackRef, kind: RefKind) -> Result<()> {
        let data = LocatedRef::Stack(self.absolute_index(index)).to_stack_data();
        let meta = self.stack_metadata_mut(index)?;
        if meta.was_moved {
            return Err(VmError::UseOfMovedValue(index));
        }
        meta.lock
            .add_lock(kind)
            .map_err(|e| VmError::LockError(e, ValueLocation::Stack(index.0)))?;
        let ref_type = PointedType::reference(meta.value_type.clone(), kind);
        let ref_meta = self.new_stack_meta_of_type(ref_type.into());
        self.stack_metadata.push(ref_meta);
        self.stack.extend(data);
        Ok(())
    }

    /// Borrows the array element at `location` through the array reference `arr_ref`
    /// and pushes a reference of `kind` to it
    pub(crate) fn push_element_ref(
        &mut self,
        arr_ref: StackRef,
        location: ValueLocation,
        element_type: VmType,
        kind: RefKind,
    ) -> Result<()> {
        let parent = self.absolute_index(arr_ref);
        let data = LocatedRef::Transient(location).to_stack_data();
        // check both locks before changing either of them
        let mut element_lock = self
            .transient_refs
            .get(&location)
            .map_or(ValueLock::None, |m| m.lock);
        element_lock
            .add_lock(kind)
            .map_err(|e| VmError::LockError(e, location))?;
        let mut parent_lock = self.absolute_metadata(parent)?.lock;
        parent_lock
            .add_element_lock()
            .map_err(|e| VmError::LockError(e, ValueLocation::Stack(arr_ref.0)))?;

        self.absolute_metadata_mut(parent)?.lock = parent_lock;
        self.transient_refs
            .entry(location)
            .or_insert_with(|| TransientMeta {
                value_type: element_type.clone(),
                lock: ValueLock::None,
            })
            .lock = element_lock;

        let ref_type = RefType {
            kind,
            pointer: element_type,
        };
        let mut meta = self.new_stack_meta_of_type(ref_type.into());
        meta.borrowed_through = Some(parent);
        self.stack_metadata.push(meta);
        self.stack.extend(data);
        Ok(())
    }

    /// Called when the value at absolute `index` stops existing (popped or overwritten)
    ///
    /// Releases the borrow held by the value, unless the value was moved out
    fn drop_value(&mut self, index: usize) -> Result<()> {
        let meta = self.absolute_metadata(index)?;
        if meta.was_moved {
            return Ok(());
        }
        let r = match meta.value_type.pointed() {
            Some(PointedType::Ref(r)) => r,
            Some(PointedType::Boxed(_)) => unimplemented!(),
            Some(PointedType::SArr(_)) | None => return Ok(()),
        };
        let kind = r.kind;
        let located = self.located(meta)?;
        let parent = meta.borrowed_through;
        match located {
            LocatedRef::Stack(target) => self
                .absolute_metadata_mut(target)?
                .lock
                .release_lock(kind)
                .map_err(|_| VmError::BadVmState),
            LocatedRef::Transient(location) => {
                let element = self
                    .transient_refs
                    .get_mut(&location)
                    .ok_or(VmError::BadVmState)?;
                element
                    .lock
                    .release_lock(kind)
                    .map_err(|_| VmError::BadVmState)?;
                if !element.lock.is_locked() {
                    self.transient_refs.remove(&location);
                }
                let parent = parent.ok_or(VmError::BadVmState)?;
                self.absolute_metadata_mut(parent)?
                    .lock
                    .release_element_lock()
                    .map_err(|_| VmError::BadVmState)
            }
        }
    }

    /// Takes the borrow needed by a copy of the value at absolute `index`
    ///
    /// Only shared references hold borrows and can be copied
    fn duplicate_borrow(&mut self, index: usize) -> Result<()> {
        let meta = self.absolute_metadata(index)?;
        let r = match meta.value_type.ref_type() {
            Some(r) => r,
            None if meta.value_type.holds_borrow() => return Err(VmError::BadVmState),
            None => return Ok(()),
        };
        if r.kind != RefKind::Ref {
            return Err(VmError::BadVmState);
        }
        let located = self.located(meta)?;
        let parent = meta.borrowed_through;
        match located {
            LocatedRef::Stack(target) => self
                .absolute_metadata_mut(target)?
                .lock
                .add_lock(RefKind::Ref)
                .map_err(|_| VmError::BadVmState),
            LocatedRef::Transient(location) => {
                self.transient_refs
                    .get_mut(&location)
                    .ok_or(VmError::BadVmState)?
                    .lock
                    .add_lock(RefKind::Ref)
                    .map_err(|_| VmError::BadVmState)?;
                let parent = parent.ok_or(VmError::BadVmState)?;
                self.absolute_metadata_mut(parent)?
                    .lock
                    .add_element_lock()
                    .map_err(|_| VmError::BadVmState)
            }
        }
    }

    /// What the reference with the metadata `meta` points to
    fn located(&self, meta: &StackMeta) -> Result<LocatedRef> {
        let data = self
            .stack
            .get(get_stack_range(meta.index.0, meta))
            .ok_or(VmError::BadVmState)?;
        LocatedRef::from_stack_data(data)
    }

    /// Absolute index of the value that must outlive the value at absolute `index`
    ///
    /// For a reference it is the value it points to, or the array reference an element was
    /// borrowed through.
    fn borrow_anchor(&self, index: usize) -> Result<Option<usize>> {
        let meta = self.absolute_metadata(index)?;
        if meta.borrowed_through.is_some() {
            return Ok(meta.borrowed_through);
        }
        match meta.value_type.ref_type() {
            Some(_) => match self.located(meta)? {
                LocatedRef::Stack(target) => Ok(Some(target)),
                LocatedRef::Transient(_) => Err(VmError::BadVmState),
            },
            None if meta.value_type.holds_borrow() => Err(VmError::BadVmState),
            None => Ok(None),
        }
    }

    /// Moves the value at `op` into `result`, a `Copy` value is copied instead
    ///
    /// The previous value of `result` is dropped. Both values must be of the same type.
    pub(crate) fn move_into(&mut self, result: StackRef, op: StackRef) -> Result<()> {
        let result_meta = self.checked_stack_metadata(result, Access::Write)?;
        let op_meta = self.checked_stack_metadata(op, Access::Move)?;
        if result_meta.value_type != op_meta.value_type {
            return Err(VmError::BadVmState);
        }
        if result == op {
            return Ok(());
        }
        let is_copy = op_meta.value_type.is_copy();
        let borrowed_through = op_meta.borrowed_through;
        let result_range = get_stack_range(result_meta.index.0, result_meta);
        let (result_index, op_index) = (self.absolute_index(result), self.absolute_index(op));
        if let Some(anchor) = self.borrow_anchor(op_index)?
            && anchor >= result_index
        {
            return Err(VmError::ReferenceEscapesScope(result));
        }

        self.drop_value(result_index)?;
        if is_copy {
            self.duplicate_borrow(op_index)?;
        }
        let value: SmallVec<[StackData; 2]> = self.stack_data(op)?.into();
        self.stack
            .get_mut(result_range)
            .ok_or(VmError::BadVmState)?
            .copy_from_slice(&value);
        let result_meta = self.stack_metadata_mut(result)?;
        result_meta.was_moved = false;
        result_meta.borrowed_through = borrowed_through;
        if !is_copy {
            self.stack_metadata_mut(op)?.was_moved = true;
        }
        Ok(())
    }

    /// Pushes the value at `op` on top of the stack, moving it out of `op` unless it is `Copy`
    pub(crate) fn push_moved(&mut self, op: StackRef) -> Result<()> {
        let meta = self.checked_stack_metadata(op, Access::Move)?;
        let t = meta.value_type.clone();
        let is_copy = t.is_copy();
        let borrowed_through = meta.borrowed_through;
        let value: SmallVec<[StackData; 2]> = self.stack_data(op)?.into();

        // the new value is above everything that exists, so it cannot outlive its borrow
        if is_copy {
            self.duplicate_borrow(self.absolute_index(op))?;
        }
        let mut meta = self.new_stack_meta_of_type(t);
        meta.borrowed_through = borrowed_through;
        self.stack_metadata.push(meta);
        self.stack.extend(value);
        if !is_copy {
            self.stack_metadata_mut(op)?.was_moved = true;
        }
        Ok(())
    }

    /// Pop the last stack value in its entirety from the stack
    pub(crate) fn pop_stack(&mut self) -> Result<()> {
        let Some(index) = self.stack_metadata.len().checked_sub(1) else {
            return Ok(());
        };
        if self.derefs.iter().any(|d| d.deref >= index) {
            return Err(VmError::ScopeEndedDuringDeref);
        }
        if self.stack_metadata[index].lock.is_locked() {
            // a borrow would outlive its value
            return Err(VmError::BadVmState);
        }
        self.drop_value(index)?;
        let meta = self.stack_metadata.pop().ok_or(VmError::BadVmState)?;
        self.stack.truncate(meta.index.0);
        Ok(())
    }

    pub(crate) fn push_scope(&mut self) -> Result<()> {
        self.cycle = self.cycle.checked_add(1).ok_or(VmError::BadVmState)?;
        Ok(())
    }

    pub(crate) fn pop_scope(&mut self) -> Result<()> {
        // a function cannot end the scopes of its caller
        let first_cycle = self.frames.last().map_or(1, |f| f.caller_cycle + 1);
        if self.cycle <= first_cycle {
            Err(VmError::BadVmState)
        } else {
            self.cycle -= 1;
            Ok(())
        }
    }

    /// Pops all the values created in the current scope and leaves it
    pub(crate) fn end_scope(&mut self) -> Result<()> {
        let cycle = self.current_cycle();
        let first_in_scope = self
            .stack_metadata
            .iter()
            .rposition(|m| m.cycle < cycle)
            .map_or(0, |i| i + 1);
        if self.derefs.iter().any(|d| d.deref >= first_in_scope) {
            return Err(VmError::ScopeEndedDuringDeref);
        }
        while self.stack_metadata.len() > first_in_scope {
            self.pop_stack()?;
        }
        self.pop_scope()
    }

    pub fn new_stack_meta_of_type(&self, t: VmType) -> StackMeta {
        let cycle = self.current_cycle();
        let len = self.stack.len();
        StackMeta::new(t, StackDataRef(len), cycle)
    }

    /// Type and stack data range of the value that the reference at absolute `index` points to
    fn pointee(&self, index: usize) -> Result<(VmType, Range<usize>)> {
        let meta = self.absolute_metadata(index)?;
        if meta.value_type.ref_type().is_none() {
            return Err(VmError::BadVmState);
        }
        match self.located(meta)? {
            LocatedRef::Stack(target) => {
                let target = self.absolute_metadata(target)?;
                let range = get_stack_range(target.index.0, target);
                Ok((target.value_type.clone(), range))
            }
            LocatedRef::Transient(location) => {
                let target = self
                    .transient_refs
                    .get(&location)
                    .ok_or(VmError::BadVmState)?;
                match location {
                    ValueLocation::Stack(start) => {
                        Ok((target.value_type.clone(), get_stack_range(start, target)))
                    }
                    ValueLocation::Heap(_) => unimplemented!(),
                }
            }
        }
    }

    /// Starts a deref of the reference at `rf`
    ///
    /// Pushes a copy of the pointed-to value, which [`pop_deref`](Self::pop_deref) writes back
    /// if the reference is mutable. The reference stays borrowed until then.
    pub(crate) fn push_deref(&mut self, rf: StackRef) -> Result<()> {
        let meta = self.checked_stack_metadata(rf, Access::Read)?;
        let kind = match meta.value_type.ref_type() {
            Some(r) => r.kind,
            None => {
                let t = meta.value_type.clone().tag("rf");
                return Err(VmError::InvalidTypeForOperation(t));
            }
        };
        let rf_index = self.absolute_index(rf);
        let (t, range) = self.pointee(rf_index)?;
        if t.holds_borrow() {
            // a copy of the value would duplicate the borrows it holds
            return Err(VmError::DerefOfBorrowingValue(rf));
        }
        let value: SmallVec<[StackData; 2]> =
            self.stack.get(range).ok_or(VmError::BadVmState)?.into();
        self.stack_metadata_mut(rf)?
            .lock
            .add_lock(kind)
            .map_err(|e| VmError::LockError(e, ValueLocation::Stack(rf.0)))?;

        let deref_index = self.stack_metadata.len();
        let mut meta = self.new_stack_meta_of_type(t);
        meta.deref = kind.into();
        self.stack_metadata.push(meta);
        self.stack.extend(value);
        self.derefs.push(VmDeref {
            rf: rf_index,
            deref: deref_index,
        });
        Ok(())
    }

    pub(crate) fn pop_deref(&mut self) -> Result<()> {
        // a function cannot end the derefs of its caller
        if !self
            .derefs
            .last()
            .is_some_and(|d| d.deref >= self.last_stack_frame)
        {
            return Err(VmError::BadVmState);
        }
        let d = self.derefs.pop().ok_or(VmError::BadVmState)?;
        let rf_meta = self.absolute_metadata(d.rf)?;
        let kind = rf_meta.value_type.ref_type().ok_or(VmError::BadVmState)?.kind;
        let deref_meta = self.absolute_metadata(d.deref)?;
        if deref_meta.deref == DerefLock::None {
            return Err(VmError::BadVmState);
        }
        if kind == RefKind::Mut {
            let deref_range = get_stack_range(deref_meta.index.0, deref_meta);
            let (_, range) = self.pointee(d.rf)?;
            if range.len() != deref_range.len() {
                return Err(VmError::BadVmState);
            }
            self.stack.copy_within(deref_range, range.start);
            self.absolute_metadata_mut(d.deref)?.was_moved = true;
        }
        self.absolute_metadata_mut(d.rf)?
            .lock
            .release_lock(kind)
            .map_err(|_| VmError::BadVmState)
    }

    /// Calls the function `f` refers to, with the top values of the stack as its arguments
    ///
    /// The arguments become the first values of the function's stack frame, created in a new
    /// scope so that the function can borrow them. Execution continues at the start of the
    /// function, [`ret`](Self::ret) continues at `return_ip`.
    ///
    /// A native function is run to completion, the call returns with execution at `return_ip`.
    pub(crate) fn call(&mut self, f: PoolRef, return_ip: usize) -> Result<()> {
        let (module, name) = self
            .current_const_pool()
            .get_fn(f)
            .ok_or(VmError::ConstantPoolError)?;
        let module = module.filter(|&m| m != self.current_module).map(str::to_owned);
        let module_name = module.as_deref().unwrap_or(&self.current_module);
        let function = self
            .modules
            .get(module_name)
            .and_then(|m| m.function(name))
            .ok_or_else(|| VmError::UnknownFunction {
                module: module_name.to_owned(),
                name: name.to_owned(),
            })?;
        let function = Rc::clone(function);

        let params = function.signature.params();
        let in_frame = self.stack_metadata.len() - self.last_stack_frame;
        if in_frame < params.len() {
            return Err(VmError::NotEnoughArguments {
                expected: params.len(),
                got: in_frame,
            });
        }
        let base = self.stack_metadata.len() - params.len();
        if self.derefs.iter().any(|d| d.deref >= base) {
            return Err(VmError::DerefPassedToCall);
        }
        let mut errors = Vec::new();
        for (i, param) in params.iter().enumerate() {
            let meta = self.absolute_metadata(base + i)?;
            if meta.was_moved {
                return Err(VmError::UseOfMovedValue(StackRef(base + i - self.last_stack_frame)));
            }
            if meta.value_type != *param {
                let arg = meta.value_type.tag(format!("arg{i}"));
                errors.push(TypeError::NotEquals(arg, param.clone()));
            }
        }
        if !errors.is_empty() {
            return Err(VmError::TypeError(errors));
        }

        let cycle = self.cycle.checked_add(1).ok_or(VmError::BadVmState)?;
        let caller_module = module.map(|m| std::mem::replace(&mut self.current_module, m));
        self.frames.push(CallFrame {
            function: Rc::clone(&function),
            return_ip,
            caller_frame: self.last_stack_frame,
            caller_cycle: self.cycle,
            caller_module,
        });
        self.last_stack_frame = base;
        self.cycle = cycle;
        self.ip = 0;
        match &function.body {
            FunctionBody::Bytecode(_) => Ok(()),
            FunctionBody::Native(f) => self.run_native(f, params.len()),
        }
    }

    /// Runs the native function `f` in the current stack frame and returns the values it pushed
    fn run_native(&mut self, f: &native::NativeFn, params: usize) -> Result<()> {
        f.call(&mut native::NativeCall::new(self, params))?;
        // the native function can only push values, they all follow the arguments
        let count = self.stack_metadata.len() - self.last_stack_frame - params;
        self.ret_values(StackRef(params), count)
    }

    /// Returns `value` from the current function to its caller
    pub(crate) fn ret(&mut self, value: StackRef) -> Result<()> {
        self.ret_values(value, 1)
    }

    /// Returns the `count` values starting at `first` from the current function to its caller
    ///
    /// Pops the function's stack frame, then pushes the values onto the caller's stack in order.
    pub(crate) fn ret_values(&mut self, first: StackRef, count: usize) -> Result<()> {
        let frame = self.frames.last().ok_or(VmError::RetOutsideFunction)?;
        let returns = frame.function.signature.returns();
        if count != returns.len() {
            return Err(VmError::WrongReturnCount {
                expected: returns.len(),
                got: count,
            });
        }
        let base = self.last_stack_frame;
        if self.derefs.iter().any(|d| d.deref >= base) {
            return Err(VmError::ScopeEndedDuringDeref);
        }

        // a single value is kept inline
        let mut values: SmallVec<[(VmType, Option<usize>); 1]> = SmallVec::new();
        let mut data: SmallVec<[StackData; 2]> = SmallVec::new();
        let mut errors = Vec::new();
        for (i, return_type) in returns.iter().enumerate() {
            let value = StackRef(first.0.checked_add(i).ok_or(VmError::BadVmState)?);
            let meta = self.checked_stack_metadata(value, Access::Move)?;
            if meta.value_type != *return_type {
                let tag = if count == 1 { "ret".to_owned() } else { format!("ret{i}") };
                errors.push(TypeError::NotEquals(meta.value_type.tag(tag), return_type.clone()));
                continue;
            }
            if let Some(anchor) = self.borrow_anchor(self.absolute_index(value))?
                && anchor >= base
            {
                return Err(VmError::ReferenceEscapesScope(value));
            }
            values.push((meta.value_type.clone(), meta.borrowed_through));
            data.extend_from_slice(self.stack_data(value)?);
        }
        if !errors.is_empty() {
            return Err(VmError::TypeError(errors));
        }

        // the slots are popped below, so even `Copy` values are moved out along with their borrows
        for i in 0..count {
            self.stack_metadata_mut(StackRef(first.0 + i))?.was_moved = true;
        }
        while self.stack_metadata.len() > base {
            self.pop_stack()?;
        }

        let frame = self.frames.pop().ok_or(VmError::BadVmState)?;
        self.last_stack_frame = frame.caller_frame;
        self.cycle = frame.caller_cycle;
        if let Some(m) = frame.caller_module {
            self.current_module = m;
        }
        self.ip = frame.return_ip;

        let mut data = data.into_iter();
        for (t, borrowed_through) in values {
            let size = t.size();
            let mut meta = self.new_stack_meta_of_type(t);
            meta.borrowed_through = borrowed_through;
            self.stack_metadata.push(meta);
            self.stack.extend(data.by_ref().take(size));
        }
        Ok(())
    }

    /// The function being executed, `None` for the top level code
    pub fn current_function(&self) -> Option<&Rc<Function>> {
        self.frames.last().map(|f| &f.function)
    }

    pub fn locate_ref(&self, index: StackRef) -> Result<(LocatedRef, &RefType)> {
        let meta = self.stack_metadata(index)?;
        if let Some(PointedType::Ref(r)) = meta.value_type.pointed() {
            let data = self.stack_data(index)?;
            Ok((LocatedRef::from_stack_data(data)?, r))
        } else {
            Err(VmError::BadVmState)
        }
    }

    pub(crate) fn push_array_0(&mut self, size: usize, t: PrimitiveType) {
        let arr_type = PointedType::s_arr(t, size);
        let stack_size = arr_type.size();
        let meta = self.new_stack_meta_of_type(arr_type.into());
        self.stack_metadata.push(meta);
        self.stack
            .extend(std::iter::repeat_n(StackData::default(), stack_size))
    }

    pub(crate) fn push_s_str(&mut self, ptr: usize, len: usize) {
        let meta = self.new_stack_meta_of_type(PrimitiveType::SStr.into());
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
            frames: Vec::new(),
        }
    }
}

/// The string the stack data of a [`SStr`](PrimitiveType::SStr) value points to
pub(crate) fn static_str(data: &[StackData]) -> Result<&str> {
    let &[ptr, len] = data else {
        return Err(VmError::BadVmState);
    };
    let ptr = usize::from_single(ptr) as *const u8;
    let len = usize::from_single(len);
    // SAFETY: a `SStr` value is only created by `LdSS` from a string of a constant pool
    // (or by a native function from a `&'static str`) and copied as a whole, it is never zeroed.
    // The pools of the loaded modules are never changed, and the modules are never removed
    // or replaced, so the string outlives the vm and with it the borrow of `data`.
    unsafe { Ok(std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))) }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum ValueLocation {
    Stack(usize),
    Heap(*const ()),
}

impl From<StackDataRef> for ValueLocation {
    fn from(obj: StackDataRef) -> Self {
        ValueLocation::Stack(obj.0)
    }
}

/// An active deref, both indexes are absolute
#[derive(Debug)]
pub struct VmDeref {
    /// The reference being dereferenced
    pub rf: usize,
    /// The copy of the pointed-to value
    pub deref: usize,
}

/// An active function call
#[derive(Debug)]
pub struct CallFrame {
    /// The called function
    pub function: Rc<Function>,
    /// Instruction of the caller to continue at after the return
    pub return_ip: usize,
    /// Stack frame of the caller
    pub caller_frame: usize,
    /// Cycle of the caller
    pub caller_cycle: usize,
    /// Module of the caller, if the called function is in another module
    pub caller_module: Option<String>,
}
