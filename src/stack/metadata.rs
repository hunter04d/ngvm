use crate::types::checker::{Tag, TypeChecker, TypeCheckerCtx};
use crate::types::VmType;
use crate::vm::lock::ValueLock;
use crate::vm::StackDataRef;

#[derive(Debug)]
pub struct StackMetadata {
    pub value_type: VmType,
    pub index: StackDataRef,
    pub cycle: usize,
    pub lock: ValueLock,
    // TODO: other meta fields
}

impl StackMetadata {
    pub fn check<'a, 'c>(
        &'a self,
        tag: impl Into<Tag>,
        ctx: &'c mut TypeCheckerCtx,
    ) -> TypeChecker<'a, 'c> {
        TypeChecker {
            tag: tag.into(),
            vm_type: &self.value_type,
            ctx,
        }
    }
}
