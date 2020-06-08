use super::{HasTypeCheckerCtx, Tag, TypeChecker, TypeCheckerCtx, TypeError};
use crate::types::checker::Taggable;
use crate::types::{PointedType, RefType, SArrType, VmType};

pub struct RefTypeChecker<'a, C: HasTypeCheckerCtx> {
    pub(super) ref_type: Option<&'a RefType>,
    pub(super) tag: Tag,
    pub(super) ctx: C,
}

pub struct SArrTypeChecker<'a, C: HasTypeCheckerCtx> {
    pub(super) arr_type: Option<&'a SArrType>,
    pub(super) tag: Tag,
    pub(super) ctx: C,
}

impl<'a, C: HasTypeCheckerCtx> RefTypeChecker<'a, C> {
    pub fn to(self) -> TypeChecker<'a, Self> {
        TypeChecker {
            tag: self.tag.clone(),
            vm_type: self.ref_type.map(|r| &r.pointer),
            ctx: self,
        }
    }
}

impl<'a, C: HasTypeCheckerCtx> SArrTypeChecker<'a, C> {
    pub fn of_type(self) -> TypeChecker<'a, Self> {
        TypeChecker {
            tag: self.tag.clone(),
            vm_type: self.arr_type.map(|r| &r.pointer),
            ctx: self,
        }
    }

    pub fn with_len(
        mut self,
        len_cond: impl Fn(usize) -> bool,
        format: impl FnOnce(&SArrType, &Tag) -> String,
    ) -> Self {
        if let Some(arr) = self.arr_type {
            if !len_cond(arr.len) {
                let fmt = format(arr, &self.tag);
                let vm_type = VmType::from(PointedType::SArr(arr.clone()));
                self.report(TypeError::Condition(vm_type.tag(self.tag.clone()), fmt))
            }
        }
        self
    }
}

impl<'a, C: HasTypeCheckerCtx> HasTypeCheckerCtx for RefTypeChecker<'a, C> {
    type Unwrapped = &'a RefType;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.root_ctx()
    }

    fn unwrap(self) -> Self::Unwrapped {
        self.ref_type.unwrap()
    }
}

impl<'a, C: HasTypeCheckerCtx> HasTypeCheckerCtx for SArrTypeChecker<'a, C> {
    type Unwrapped = &'a SArrType;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.root_ctx()
    }

    fn unwrap(self) -> Self::Unwrapped {
        self.arr_type.unwrap()
    }
}
