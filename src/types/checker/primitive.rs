use crate::types::{PrimitiveType, ThreePrimitiveTypes, TwoPrimitiveTypes, VmType};

use super::{tags, HasTypeCheckerCtx, Tag, Taggable, TaggedType, TypeCheckerCtx, TypeError};

fn report_not_user(t: &TaggedType) -> String {
    format!("{} of {:?} is not a user primitive type", t.tag, t.vm_type)
}

pub struct PrimitiveTypeChecker<C: HasTypeCheckerCtx> {
    tag: Tag,
    t: Option<PrimitiveType>,
    ctx: C,
}

impl<C: HasTypeCheckerCtx> PrimitiveTypeChecker<C> {
    fn report_if(
        mut self,
        cond: impl FnOnce(PrimitiveType) -> bool,
        report: impl FnOnce(TaggedType) -> TypeError,
    ) -> C {
        match self.t {
            Some(p) if cond(p) => {
                let tagged = VmType::Primitive(p).tag(self.tag);
                let err = report(tagged);
                self.ctx.ctx().report(err);
            }
            _ => {}
        }
        self.ctx
    }

    pub fn one_of(self, types: &[PrimitiveType]) -> C {
        self.report_if(
            |t| !types.contains(&t),
            |t| TypeError::NotOneOf(t, types.iter().map(|&p| VmType::Primitive(p)).collect()),
        )
    }

    pub fn equals(self, p: PrimitiveType) -> C {
        self.report_if(
            |t| t != p,
            |t| TypeError::NotEquals(t, VmType::Primitive(p)),
        )
    }

    pub fn cond(
        self,
        cond: impl FnOnce(PrimitiveType) -> bool,
        format: impl FnOnce(&TaggedType) -> String,
    ) -> C {
        self.report_if(
            |t| !cond(t),
            |t| {
                let formatted = format(&t);
                TypeError::Condition(t, formatted)
            },
        )
    }

    pub fn user(self) -> C {
        self.cond(|t| t.is_user(), report_not_user)
    }

    pub fn integer(self) -> C {
        self.cond(
            |t| t.is_user(),
            |t| format!("<{}>{:?} is not integer", t.tag, t.vm_type),
        )
    }

    pub fn bool(self) -> C {
        self.equals(PrimitiveType::Bool)
    }

    pub fn float(self) -> C {
        self.one_of(&[PrimitiveType::F64, PrimitiveType::F32])
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
            self.ctx.ctx().report(TypeError::AllNotEqual(errors))
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

    fn ctx(&mut self) -> &mut TypeCheckerCtx {
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
                self.ctx().report(TypeError::ThreeNotEqual(
                    VmType::Primitive(result).tag(tags::RESULT),
                    VmType::Primitive(op1).tag(tags::OP1),
                    VmType::Primitive(op2).tag(tags::OP2),
                ));
            }
        }
        AllSamePrimitiveTypeChecker {
            tag: format!("{}, {}, {}", tags::RESULT, tags::OP1, tags::OP2).into(),
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

    fn ctx(&mut self) -> &mut TypeCheckerCtx {
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
                self.ctx().report(TypeError::TwoNotEqual(
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

    fn ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.ctx()
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

    fn ctx(&mut self) -> &mut TypeCheckerCtx {
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
                 self.ctx().report(TypeError::TwoNotEqual(
                     VmType::Primitive(op1).tag(tags::OP1),
                     VmType::Primitive(op2).tag(tags::OP2)));
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
        format: impl Fn(&TaggedType) -> String,
    ) -> ThreePrimitiveTypesChecker<'c> {
        self.op1()
            .cond(|t| cond(t), |t| format(t))
            .op2()
            .cond(|t| cond(t), |t| format(t))
            .ctx
    }
}

impl HasTypeCheckerCtx for PrimitiveOperandsTypeChecker<'_> {
    type Unwrapped = ThreePrimitiveTypes;

    fn ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx.ctx()
    }

    fn unwrap(self) -> Self::Unwrapped {
        ThreePrimitiveTypes {
            result: self.ctx.result.unwrap(),
            op1: self.op1.unwrap(),
            op2: self.op2.unwrap(),
        }
    }
}
