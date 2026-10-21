use crate::types::{PrimitiveType, VmType};

use super::{Checked, HasTypeCheckerCtx, Tag, Taggable, TaggedType, TypeCheckerCtx, TypeError};

pub struct PrimitiveTaggedType {
    pub tag: Tag,
    pub t: PrimitiveType,
}

fn report_not_user(t: &PrimitiveTaggedType) -> String {
    format!("{} of {:?} is not a user primitive type", t.tag, t.t)
}

pub struct PrimitiveTypeChecker<C: HasTypeCheckerCtx> {
    pub(super) tag: Tag,
    pub(super) t: Checked<PrimitiveType>,
    pub(super) ctx: C,
}

impl<C: HasTypeCheckerCtx> PrimitiveTypeChecker<C> {
    fn report_if(
        mut self,
        cond: impl FnOnce(PrimitiveType) -> bool,
        report: impl FnOnce(PrimitiveTaggedType) -> TypeError,
    ) -> Self {
        match self.t.value() {
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

    pub fn integer(self) -> Self {
        self.cond(
            |t| t.is_integer(),
            |t| format!("<{}>{:?} is not integer", t.tag, t.t),
        )
    }

    pub fn number(self) -> Self {
        self.cond(
            |t| t.is_number(),
            |t| format!("<{}>{:?} is not a number", t.tag, t.t),
        )
    }

    pub fn unsigned(self) -> Self {
        self.cond(
            |t| t.is_unsigned(),
            |t| format!("<{}>{:?} is not an unsigned integer", t.tag, t.t),
        )
    }

    pub fn signed(self) -> Self {
        self.cond(
            |t| t.is_signed(),
            |t| format!("<{}>{:?} is not a signed integer", t.tag, t.t),
        )
    }

    pub fn either(self) -> PrimitiveTypeChecker<PrimitiveEitherTypeChecker<C>> {
        let ctx = self.ctx;
        let either = PrimitiveEitherTypeChecker {
            tag: self.tag.clone(),
            t: self.t,
            ctx,
            error: None,
            passed: false,
        };
        PrimitiveTypeChecker {
            tag: self.tag,
            t: self.t,
            ctx: either,
        }
    }

    pub fn bool(self) -> Self {
        self.equals(PrimitiveType::Bool)
    }

    pub fn float(self) -> Self {
        self.one_of(&[PrimitiveType::F64, PrimitiveType::F32])
    }

    pub fn and(self) -> C {
        self.ctx
    }
}

impl<C: HasTypeCheckerCtx> HasTypeCheckerCtx for PrimitiveTypeChecker<C> {
    type Unwrapped = PrimitiveType;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.root_ctx()
    }

    fn unwrap(self) -> Self::Unwrapped {
        self.t.unwrap()
    }
}

impl<C: HasTypeCheckerCtx> PrimitiveTypeChecker<PrimitiveEitherTypeChecker<C>> {
    /// Starts the next alternative: if the previous one failed, its error is discarded, otherwise
    /// the errors of the remaining alternatives are ignored
    pub fn or(mut self) -> PrimitiveTypeChecker<PrimitiveEitherTypeChecker<C>> {
        if self.ctx.error.take().is_none() {
            self.ctx.passed = true;
        }
        self
    }
}

pub struct PrimitiveEitherTypeChecker<C: HasTypeCheckerCtx> {
    tag: Tag,
    t: Checked<PrimitiveType>,
    ctx: C,
    error: Option<TypeError>,
    /// One of the alternatives has already passed
    passed: bool,
}

impl<C: HasTypeCheckerCtx> PrimitiveEitherTypeChecker<C> {
    pub fn and(mut self) -> C {
        if let Some(e) = self.error {
            self.ctx.root_ctx().report(e)
        }
        self.ctx
    }

    pub fn fmt(mut self, format: impl FnOnce(&TypeError, &TaggedType) -> String) -> C {
        if let (Some(e), Some(t)) = (self.error, self.t.value()) {
            let tagged = VmType::from(t).tag(self.tag);
            let format = format(&e, &tagged);
            self.ctx
                .root_ctx()
                .report(TypeError::From(Box::new(e), format));
        }
        self.ctx
    }
}

impl<C: HasTypeCheckerCtx> HasTypeCheckerCtx for PrimitiveEitherTypeChecker<C> {
    type Unwrapped = ();

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.root_ctx()
    }

    fn unwrap(self) -> Self::Unwrapped {}

    fn report(&mut self, e: TypeError) {
        if !self.passed {
            self.error = Some(e);
        }
    }
}
