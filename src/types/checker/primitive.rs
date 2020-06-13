use crate::types::{PrimitiveType, ThreePrimitiveTypes, TwoPrimitiveTypes, VmType};

use super::{tags, HasTypeCheckerCtx, Tag, Taggable, TaggedType, TypeCheckerCtx, TypeError};

pub struct PrimitiveTaggedType {
    pub tag: Tag,
    pub t: PrimitiveType,
}

fn report_not_user(t: &PrimitiveTaggedType) -> String {
    format!("{} of {:?} is not a user primitive type", t.tag, t.t)
}

pub struct PrimitiveTypeChecker<C: HasTypeCheckerCtx> {
    pub(super) tag: Tag,
    pub(super) t: Option<PrimitiveType>,
    pub(super) ctx: C,
}

impl<C: HasTypeCheckerCtx> PrimitiveTypeChecker<C> {
    fn report_if(
        mut self,
        cond: impl FnOnce(PrimitiveType) -> bool,
        report: impl FnOnce(PrimitiveTaggedType) -> TypeError,
    ) -> Self {
        match self.t {
            Some(p) if cond(p) => {
                let tagged = PrimitiveTaggedType {
                    t: p,
                    tag: self.tag.clone(),
                };
                let err = report(tagged);

                self.ctx.root_ctx().report(err);
            }
            _ => {}
        }
        PrimitiveTypeChecker {
            tag: self.tag,
            t: self.t,
            ctx: self.ctx,
        }
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

    pub fn unsigned(self) -> Self {
        self.cond(
            |t| t.is_unsigned(),
            |t| format!("<{}>{:?} is not an unsigned integer", t.tag, t.t),
        )
    }

    pub fn signed(self) -> Self {
        self.cond(
            |t| t.is_signed(),
            |t| format!("<{}>{:?} is not an unsigned integer", t.tag, t.t),
        )
    }

    pub fn either(self) -> PrimitiveTypeChecker<PrimitiveEitherTypeChecker<C>> {
        let ctx = self.ctx;
        let either = PrimitiveEitherTypeChecker {
            tag: self.tag.clone(),
            t: self.t,
            ctx,
            error: None,
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

    pub fn along_with(
        self,
        other: impl FnOnce(C) -> PrimitiveTypeChecker<C>,
    ) -> CombinedPrimitiveTypesChecker<C> {
        let ctx = self.ctx;
        let other_checker = other(ctx);
        CombinedPrimitiveTypesChecker {
            types: vec![(self.tag, self.t), (other_checker.tag, other_checker.t)],
            ctx: other_checker.ctx,
        }
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
    pub fn or(self) -> PrimitiveTypeChecker<PrimitiveEitherTypeChecker<C>> {
        PrimitiveTypeChecker {
            tag: self.tag,
            t: self.ctx.error.as_ref().and(self.t),
            ctx: self.ctx,
        }
    }
}

pub struct PrimitiveEitherTypeChecker<C: HasTypeCheckerCtx> {
    tag: Tag,
    t: Option<PrimitiveType>,
    ctx: C,
    error: Option<TypeError>,
}

impl<C: HasTypeCheckerCtx> PrimitiveEitherTypeChecker<C> {
    pub fn and(mut self) -> C {
        if let Some(e) = self.error {
            self.ctx.root_ctx().report(e)
        }
        self.ctx
    }

    pub fn fmt(mut self, format: impl FnOnce(&TypeError, &TaggedType) -> String) -> C {
        if let (Some(e), Some(t)) = (self.error, self.t) {
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
        self.error = Some(e);
    }
}

pub struct CombinedPrimitiveTypesChecker<C: HasTypeCheckerCtx> {
    types: Vec<(Tag, Option<PrimitiveType>)>,
    ctx: C,
}

impl<C: HasTypeCheckerCtx> CombinedPrimitiveTypesChecker<C> {
    pub fn and_with(self, other: impl FnOnce(C) -> PrimitiveTypeChecker<C>) -> Self {
        let CombinedPrimitiveTypesChecker { ctx, mut types } = self;
        let other_checker = other(ctx);
        types.push((other_checker.tag, other_checker.t));
        Self {
            types,
            ctx: other_checker.ctx,
        }
    }

    pub fn are(
        mut self,
        cond: impl Fn(PrimitiveTypeChecker<TypeCheckerCtx>) -> PrimitiveTypeChecker<TypeCheckerCtx>,
    ) -> C {
        for (tag, t) in &mut self.types {
            if let Some(to_check) = t {
                let ctx = TypeCheckerCtx::new();
                let ch = PrimitiveTypeChecker {
                    tag: (*tag).clone(),
                    t: Some(*to_check),
                    ctx,
                };
                let ctx = cond(ch).and();
                for e in ctx.errors {
                    self.ctx.report(e);
                }
            }
        }
        self.ctx
    }

    pub fn are_same(mut self) -> Self {
        let mut errors = Vec::new();
        self.types
            .iter()
            .filter_map(|(tag, t)| t.map(|p| (tag, p)))
            .fold(None, |state: Option<(&Tag, PrimitiveType)>, n| {
                if let Some(i) = state {
                    if i.1 != n.1 {
                        errors.push(VmType::Primitive(i.1).tag(i.0.clone()));
                    }
                }
                Some(n)
            });
        if !errors.is_empty() {
            self.ctx.root_ctx().report(TypeError::AllNotEqual(errors))
        }
        self
    }

    pub fn and(self) -> C {
        self.ctx
    }
}

pub struct AllSamePrimitiveTypeChecker<'c> {
    tag: Tag,
    pub(super) t: Option<PrimitiveType>,
    pub(super) ctx: &'c mut TypeCheckerCtx,
}

impl AllSamePrimitiveTypeChecker<'_> {
    pub fn and(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: self.tag.clone(),
            t: self.t,
            ctx: self,
        }
    }
}

impl HasTypeCheckerCtx for AllSamePrimitiveTypeChecker<'_> {
    type Unwrapped = PrimitiveType;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx
    }

    fn unwrap(self) -> Self::Unwrapped {
        self.t.unwrap()
    }
}

#[derive(Debug)]
pub struct ThreePrimitiveTypesChecker<'c> {
    pub(super) result: Option<PrimitiveType>,
    pub(super) op1: Option<PrimitiveType>,
    pub(super) op2: Option<PrimitiveType>,
    pub(super) ctx: &'c mut TypeCheckerCtx,
}

impl<'c> ThreePrimitiveTypesChecker<'c> {
    pub fn all_same(mut self) -> AllSamePrimitiveTypeChecker<'c> {
        if let (Some(result), Some(op1), Some(op2)) = (self.result, self.op1, self.op2) {
            if result != op1 || result != op2 || op1 != op2 {
                self.root_ctx().report(TypeError::ThreeNotEqual(
                    VmType::Primitive(result).tag(tags::RESULT),
                    VmType::Primitive(op1).tag(tags::OP1),
                    VmType::Primitive(op2).tag(tags::OP2),
                ));
            }
        }
        AllSamePrimitiveTypeChecker {
            tag: tags::RESULT_OP1_OP2.into(),
            t: self.result,
            ctx: self.ctx,
        }
    }

    pub fn result(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: tags::RESULT.into(),
            t: self.result,
            ctx: self,
        }
    }

    pub fn op1(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: tags::OP1.into(),
            t: self.op1,
            ctx: self,
        }
    }

    pub fn op2(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: tags::OP2.into(),
            t: self.op2,
            ctx: self,
        }
    }

    pub fn operands(self) -> PrimitiveOperandsTypeChecker<'c> {
        PrimitiveOperandsTypeChecker {
            op1: self.op1,
            op2: self.op2,
            ctx: self,
        }
    }
}

impl<'c> HasTypeCheckerCtx for ThreePrimitiveTypesChecker<'c> {
    type Unwrapped = ThreePrimitiveTypes;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx
    }

    fn unwrap(self) -> Self::Unwrapped {
        ThreePrimitiveTypes {
            result: self.result.unwrap(),
            op1: self.op1.unwrap(),
            op2: self.op2.unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct TwoPrimitiveTypesChecker<'c> {
    pub(super) result: Option<PrimitiveType>,
    pub(super) op: Option<PrimitiveType>,
    pub(super) ctx: &'c mut TypeCheckerCtx,
}

#[derive(Debug)]
pub struct TwoPrimitiveMergedTypesChecker<'c> {
    pub(super) result: Option<PrimitiveType>,
    pub(super) op: Option<PrimitiveType>,
    pub(super) ctx: &'c mut TypeCheckerCtx,
}

impl<'c> TwoPrimitiveTypesChecker<'c> {
    pub fn op(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: tags::OP.into(),
            t: self.op,
            ctx: self,
        }
    }
    pub fn result(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: tags::OP.into(),
            t: self.result,
            ctx: self,
        }
    }

    pub fn all_same(mut self) -> AllSamePrimitiveTypeChecker<'c> {
        if let (Some(result), Some(op)) = (self.result, self.op) {
            if result != op {
                self.root_ctx().report(TypeError::TwoNotEqual(
                    VmType::Primitive(result).tag(tags::RESULT),
                    VmType::Primitive(op).tag(tags::OP),
                ));
            }
        }
        AllSamePrimitiveTypeChecker {
            tag: format!("{}, {}", tags::RESULT, tags::OP).into(),
            t: self.result,
            ctx: self.ctx,
        }
    }
}

impl TwoPrimitiveMergedTypesChecker<'_> {
    #[inline]
    pub fn and(self) -> PrimitiveTypeChecker<Self> {
        self.ops()
    }
    pub fn ops(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: "op1, op2".into(),
            t: self.op,
            ctx: self,
        }
    }

    pub fn result(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: tags::OP.into(),
            t: self.result,
            ctx: self,
        }
    }
}

impl HasTypeCheckerCtx for TwoPrimitiveTypesChecker<'_> {
    type Unwrapped = TwoPrimitiveTypes;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.root_ctx()
    }

    fn unwrap(self) -> Self::Unwrapped {
        TwoPrimitiveTypes {
            result: self.result.unwrap(),
            op: self.result.unwrap(),
        }
    }
}

impl HasTypeCheckerCtx for TwoPrimitiveMergedTypesChecker<'_> {
    type Unwrapped = TwoPrimitiveTypes;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx
    }

    fn unwrap(self) -> Self::Unwrapped {
        TwoPrimitiveTypes {
            result: self.result.unwrap(),
            op: self.op.unwrap(),
        }
    }
}

pub struct PrimitiveOperandsTypeChecker<'c> {
    op1: Option<PrimitiveType>,
    op2: Option<PrimitiveType>,
    ctx: ThreePrimitiveTypesChecker<'c>,
}

impl<'c> PrimitiveOperandsTypeChecker<'c> {
    pub fn op1(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: tags::OP1.into(),
            t: self.op1,
            ctx: self,
        }
    }

    pub fn op2(self) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: tags::OP2.into(),
            t: self.op2,
            ctx: self,
        }
    }

    pub fn same(mut self) -> TwoPrimitiveMergedTypesChecker<'c> {
        if let (Some(op1), Some(op2)) = (self.op1, self.op2) {
            if op1 != op2 {
                self.root_ctx().report(TypeError::TwoNotEqual(
                    VmType::Primitive(op1).tag(tags::OP1),
                    VmType::Primitive(op2).tag(tags::OP2),
                ));
            }
        };
        TwoPrimitiveMergedTypesChecker {
            result: self.ctx.result,
            op: self.op1,
            ctx: self.ctx.ctx,
        }
    }

    pub fn both_cond(
        self,
        cond: impl Fn(PrimitiveType) -> bool,
        format: impl Fn(&PrimitiveTaggedType) -> String,
    ) -> ThreePrimitiveTypesChecker<'c> {
        self.op1()
            .cond(|t| cond(t), |t| format(t))
            .and()
            .op2()
            .cond(|t| cond(t), |t| format(t))
            .and()
            .ctx
    }
}

impl HasTypeCheckerCtx for PrimitiveOperandsTypeChecker<'_> {
    type Unwrapped = ThreePrimitiveTypes;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.root_ctx()
    }

    fn unwrap(self) -> Self::Unwrapped {
        ThreePrimitiveTypes {
            result: self.ctx.result.unwrap(),
            op1: self.op1.unwrap(),
            op2: self.op2.unwrap(),
        }
    }
}
