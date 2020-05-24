use crate::refs::StackRef;
use crate::types::checker::{Tag, TypeChecker, TypeCheckerCtx};
use crate::types::VmType;
use crate::vm::StackDataRef;

#[derive(Debug)]
pub struct StackMetadata {
    pub value_type: VmType,
    pub index: StackDataRef,
    // TODO: other meta fields
}

impl StackMetadata {
    pub fn new(value_type: VmType, index: StackRef) -> Self {
        Self {
            value_type,
            index: StackDataRef(index.0),
        }
    }

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
