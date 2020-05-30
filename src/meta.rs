use crate::types::checker::{Tag, TypeChecker, TypeCheckerCtx};
use crate::types::VmType;
use crate::vm::lock::{DerefLock, ValueLock};
use crate::vm::refs::LocatedRef;
use crate::vm::StackDataRef;

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

pub trait Meta {
    fn vm_type(&self) -> &VmType;

    fn check<'a, 'c>(
        &'a self,
        tag: impl Into<Tag>,
        ctx: &'c mut TypeCheckerCtx,
    ) -> TypeChecker<'a, 'c> {
        TypeChecker {
            tag: tag.into(),
            vm_type: self.vm_type(),
            ctx,
        }
    }
}

impl Meta for StackMeta {
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
    fn vm_type(&self) -> &VmType {
        &self.value_type
    }
}
