use crate::types::{
    FloatType, IntegerType, NumberType, PrimitiveClass, PrimitiveType, SignedType, UnsignedType,
    VmType,
};

use super::{Checked, HasTypeCheckerCtx, NestedChecker, Tag, Taggable, TypeCheckerCtx, TypeError};

pub struct PrimitiveTaggedType {
    pub tag: Tag,
    pub t: PrimitiveType,
}

fn report_not_user(t: &PrimitiveTaggedType) -> String {
    format!("{} of {:?} is not a user primitive type", t.tag, t.t)
}

/// Checks a primitive type, `T` is the type narrowed by the checks so far
pub struct PrimitiveTypeChecker<C: HasTypeCheckerCtx, T = PrimitiveType> {
    pub(super) tag: Tag,
    pub(super) t: Checked<T>,
    pub(super) ctx: C,
}

impl<C: HasTypeCheckerCtx, T: Copy + Into<PrimitiveType>> PrimitiveTypeChecker<C, T> {
    fn report_if(
        mut self,
        cond: impl FnOnce(PrimitiveType) -> bool,
        report: impl FnOnce(PrimitiveTaggedType) -> TypeError,
    ) -> Self {
        match self.t.value().map(Into::into) {
            Some(p) if cond(p) => {
                let tagged = PrimitiveTaggedType {
                    t: p,
                    tag: self.tag.clone(),
                };
                let err = report(tagged);

                self.ctx.report(err);
            }
            _ => {}
        }
        self
    }

    pub fn one_of(self, types: &[PrimitiveType]) -> Self {
        self.report_if(
            |t| !types.contains(&t),
            |t| {
                TypeError::NotOneOf(
                    t.into(),
                    types.iter().map(|&p| VmType::Primitive(p)).collect(),
                )
            },
        )
    }

    pub fn equals(self, p: PrimitiveType) -> Self {
        self.report_if(
            |t| t != p,
            |t| TypeError::NotEquals(t.into(), VmType::Primitive(p)),
        )
    }

    pub fn cond(
        self,
        cond: impl FnOnce(PrimitiveType) -> bool,
        format: impl FnOnce(&PrimitiveTaggedType) -> String,
    ) -> Self {
        self.report_if(
            |t| !cond(t),
            |t| {
                let fmt = format(&t);
                TypeError::Condition(t.into(), fmt)
            },
        )
    }

    pub fn user(self) -> Self {
        self.cond(PrimitiveType::is_user, report_not_user)
    }

    pub fn bool(self) -> Self {
        self.equals(PrimitiveType::Bool)
    }

    /// Checks that the type is a member of the class `K`, narrowing it to `K`
    pub fn is<K: PrimitiveClass>(self) -> PrimitiveTypeChecker<C, K> {
        let Self { tag, t, mut ctx } = self;
        let t = t.and_then(|t| match K::narrow(t.into()) {
            Some(k) => Checked::known(k),
            None => {
                let tagged = VmType::from(t.into()).tag(tag.clone());
                Checked::poisoned(&mut ctx, TypeError::NotOfClass(tagged, K::NAME))
            }
        });
        PrimitiveTypeChecker { tag, t, ctx }
    }

    pub fn integer(self) -> PrimitiveTypeChecker<C, IntegerType> {
        self.is()
    }

    pub fn number(self) -> PrimitiveTypeChecker<C, NumberType> {
        self.is()
    }

    pub fn unsigned(self) -> PrimitiveTypeChecker<C, UnsignedType> {
        self.is()
    }

    pub fn signed(self) -> PrimitiveTypeChecker<C, SignedType> {
        self.is()
    }

    pub fn float(self) -> PrimitiveTypeChecker<C, FloatType> {
        self.is()
    }
}

impl<C: HasTypeCheckerCtx, T> PrimitiveTypeChecker<C, T> {
    /// Ends the check of this type, continuing with the check it is nested in
    pub fn parent(self) -> C {
        self.ctx
    }
}

impl<C: HasTypeCheckerCtx, T> NestedChecker<C> for PrimitiveTypeChecker<C, T> {
    fn into_parent(self) -> C {
        self.parent()
    }
}

impl<C: HasTypeCheckerCtx, T> HasTypeCheckerCtx for PrimitiveTypeChecker<C, T> {
    type Unwrapped = T;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.root_ctx()
    }

    fn unwrap(self) -> Self::Unwrapped {
        self.t.unwrap()
    }
}
