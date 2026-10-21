use crate::types::checker::Taggable;
use crate::types::{PointedType, RefType, SArrType, VmType};

use super::{
    Checked, HasTypeCheckerCtx, NestedChecker, Tag, TypeChecker, TypeCheckerCtx, TypeError,
};

pub struct RefTypeChecker<'a, C: HasTypeCheckerCtx> {
    pub(super) ref_type: Checked<&'a RefType>,
    pub(super) tag: Tag,
    pub(super) ctx: C,
}

pub struct SArrTypeChecker<'a, C: HasTypeCheckerCtx> {
    pub(super) arr_type: Checked<&'a SArrType>,
    pub(super) tag: Tag,
    pub(super) ctx: C,
}

impl<'a, C: HasTypeCheckerCtx> RefTypeChecker<'a, C> {
    /// Runs `check` on the type the reference points to
    pub fn to<N: NestedChecker<Self>>(
        self,
        check: impl FnOnce(TypeChecker<'a, Self>) -> N,
    ) -> Self {
        let pointed = TypeChecker {
            tag: self.tag.clone(),
            vm_type: self.ref_type.map(|r| &r.pointer),
            ctx: self,
        };
        check(pointed).into_parent()
    }

    /// Ends the check of this type, continuing with the check it is nested in
    pub fn parent(self) -> C {
        self.ctx
    }
}

impl<'a, C: HasTypeCheckerCtx> SArrTypeChecker<'a, C> {
    /// Runs `check` on the type of the array elements
    pub fn of_type<N: NestedChecker<Self>>(
        self,
        check: impl FnOnce(TypeChecker<'a, Self>) -> N,
    ) -> Self {
        let element = TypeChecker {
            tag: self.tag.clone(),
            vm_type: self.arr_type.map(|r| &r.pointer),
            ctx: self,
        };
        check(element).into_parent()
    }

    pub fn with_len(
        mut self,
        len_cond: impl Fn(usize) -> bool,
        format: impl FnOnce(&SArrType, &Tag) -> String,
    ) -> Self {
        if let Some(arr) = self.arr_type.value()
            && !len_cond(arr.len)
        {
            let fmt = format(arr, &self.tag);
            let vm_type = VmType::from(PointedType::SArr(arr.clone()));
            self.report(TypeError::Condition(vm_type.tag(self.tag.clone()), fmt))
        }
        self
    }

    /// Ends the check of this type, continuing with the check it is nested in
    pub fn parent(self) -> C {
        self.ctx
    }
}

impl<C: HasTypeCheckerCtx> NestedChecker<C> for RefTypeChecker<'_, C> {
    fn into_parent(self) -> C {
        self.parent()
    }
}

impl<C: HasTypeCheckerCtx> NestedChecker<C> for SArrTypeChecker<'_, C> {
    fn into_parent(self) -> C {
        self.parent()
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
