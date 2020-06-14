use crate::types::{HasVmType, VmType};
use crate::types::checker::{Tag, TypeChecker, TypeCheckerCtx};
use crate::vm::lock::{DerefLock, ValueLock};
use crate::vm::refs::LocatedRef;
use crate::vm::StackDataRef;

pub enum VmMetaView<'a> {
    Stack(&'a StackMeta),
    Transient(&'a TransientMeta),
}

impl VmMetaView<'_> {
    pub fn lock(&self) -> &ValueLock {
        match self {
            VmMetaView::Stack(s) => s.lock(),
            VmMetaView::Transient(t) => t.lock(),
        }
    }
}

impl HasVmType for VmMetaView<'_> {
    fn vm_type(&self) -> &VmType {
        match *self {
            VmMetaView::Stack(s) => s.vm_type(),
            VmMetaView::Transient(t) => t.vm_type(),
        }
    }
}

#[derive(Debug)]
pub struct StackMeta {
    pub value_type: VmType,
    pub index: StackDataRef,
    pub cycle: usize,
    pub lock: ValueLock,
    pub was_moved: bool,
    pub deref: DerefLock,
    // TODO: other meta fields
}

impl StackMeta {
    pub fn new(value_type: impl Into<VmType>, index: StackDataRef, cycle: usize) -> Self {
        StackMeta {
            value_type: value_type.into(),
            index,
            cycle,
            lock: Default::default(),
            was_moved: false,
            deref: Default::default(),
        }
    }
}

pub trait Meta: HasVmType {
    fn lock(&self) -> &ValueLock;

    fn lock_mut(&mut self) -> &mut ValueLock;

    fn check_with<'a, 'c>(
        &'a self,
        tag: impl Into<Tag>,
        ctx: &'c mut TypeCheckerCtx,
    ) -> TypeChecker<'a, &'c mut TypeCheckerCtx> {
        TypeChecker {
            tag: tag.into(),
            vm_type: Some(self.vm_type()),
            ctx,
        }
    }

    fn check(&self, tag: impl Into<Tag>) -> TypeChecker<TypeCheckerCtx> {
        TypeChecker {
            tag: tag.into(),
            vm_type: Some(self.vm_type()),
            ctx: TypeCheckerCtx::new(),
        }
    }
}

impl Meta for StackMeta {
    fn lock(&self) -> &ValueLock {
        &self.lock
    }

    fn lock_mut(&mut self) -> &mut ValueLock {
        &mut self.lock
    }
}
impl HasVmType for StackMeta {
    fn vm_type(&self) -> &VmType {
        &self.value_type
    }
}

#[derive(Debug)]
pub struct TransientMeta {
    pub value_type: VmType,
    pub root_object: LocatedRef,
    pub lock: ValueLock,
    pub was_moved: bool,
}

impl Meta for TransientMeta {
    fn lock(&self) -> &ValueLock {
        &self.lock
    }

    fn lock_mut(&mut self) -> &mut ValueLock {
        &mut self.lock
    }
}

impl HasVmType for TransientMeta {
    fn vm_type(&self) -> &VmType {
        &self.value_type
    }
}
