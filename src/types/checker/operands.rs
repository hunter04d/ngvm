//! Checks of the operands of an instruction, each operand is named by its tag

use crate::types::{PrimitiveType, ThreePrimitiveTypes, TwoPrimitiveTypes, VmType};

use super::primitive::PrimitiveTypeChecker;
use super::{Checked, HasTypeCheckerCtx, Tag, Taggable, TypeCheckerCtx, TypeError, tags};

pub struct OperandTypesChecker<'a, 'c, const N: usize> {
    operands: [(&'static str, &'a VmType); N],
    ctx: &'c mut TypeCheckerCtx,
}

impl<'a, 'c> OperandTypesChecker<'a, 'c, 3> {
    pub fn three(
        result: &'a VmType,
        op1: &'a VmType,
        op2: &'a VmType,
        ctx: &'c mut TypeCheckerCtx,
    ) -> Self {
        Self {
            operands: [(tags::RESULT, result), (tags::OP1, op1), (tags::OP2, op2)],
            ctx,
        }
    }
}

impl<'a, 'c> OperandTypesChecker<'a, 'c, 2> {
    pub fn two(result: &'a VmType, op: &'a VmType, ctx: &'c mut TypeCheckerCtx) -> Self {
        Self {
            operands: [(tags::RESULT, result), (tags::OP, op)],
            ctx,
        }
    }
}

impl<'c, const N: usize> OperandTypesChecker<'_, 'c, N> {
    pub fn all_primitives(self) -> PrimitiveOperandsChecker<'c, N> {
        let ctx = self.ctx;
        let operands = self.operands.map(|(tag, t)| {
            let p = match t.primitive() {
                Some(p) => Checked::known(p),
                None => Checked::poisoned(&mut *ctx, TypeError::NotPrimitive(t.tag(tag))),
            };
            (tag, p)
        });
        PrimitiveOperandsChecker { operands, ctx }
    }
}

pub struct PrimitiveOperandsChecker<'c, const N: usize> {
    operands: [(&'static str, Checked<PrimitiveType>); N],
    ctx: &'c mut TypeCheckerCtx,
}

impl<const N: usize> PrimitiveOperandsChecker<'_, N>
where
    Self: HasTypeCheckerCtx,
{
    fn find(&self, tag: &str) -> Checked<PrimitiveType> {
        self.operands
            .iter()
            .find(|(t, _)| *t == tag)
            .map(|&(_, p)| p)
            .unwrap_or_else(|| panic!("the instruction has no operand <{tag}>"))
    }

    /// Checks the operand named by `tag`
    pub fn operand(self, tag: &'static str) -> PrimitiveTypeChecker<Self> {
        PrimitiveTypeChecker {
            tag: tag.into(),
            t: self.find(tag),
            ctx: self,
        }
    }

    /// Checks that the operands named by `operands` have the same type, then checks that type
    pub fn same(mut self, operands: &'static [&'static str]) -> PrimitiveTypeChecker<Self> {
        let &[first_tag, ..] = operands else {
            panic!("same() needs at least one operand");
        };
        let known = || {
            operands
                .iter()
                .filter_map(|&tag| self.find(tag).value().map(|p| (tag, p)))
        };
        let mut types = known().map(|(_, p)| p);
        if let Some(first) = types.next()
            && types.any(|p| p != first)
        {
            let tagged = known()
                .map(|(tag, p)| VmType::Primitive(p).tag(tag))
                .collect();
            self.report(TypeError::AllNotEqual(tagged));
        }
        PrimitiveTypeChecker {
            tag: Tag::Group(operands),
            t: self.find(first_tag),
            ctx: self,
        }
    }
}

impl<'c> PrimitiveOperandsChecker<'c, 3> {
    pub fn result(self) -> PrimitiveTypeChecker<Self> {
        self.operand(tags::RESULT)
    }

    pub fn op1(self) -> PrimitiveTypeChecker<Self> {
        self.operand(tags::OP1)
    }

    pub fn op2(self) -> PrimitiveTypeChecker<Self> {
        self.operand(tags::OP2)
    }

    /// Checks that both operands have the same type, then checks that type
    pub fn same_operands(self) -> PrimitiveTypeChecker<Self> {
        self.same(&[tags::OP1, tags::OP2])
    }

    /// Checks that the result and both operands have the same type, then checks that type
    pub fn all_same(self) -> PrimitiveTypeChecker<Self> {
        self.same(&[tags::RESULT, tags::OP1, tags::OP2])
    }
}

impl<'c> PrimitiveOperandsChecker<'c, 2> {
    pub fn result(self) -> PrimitiveTypeChecker<Self> {
        self.operand(tags::RESULT)
    }

    pub fn op(self) -> PrimitiveTypeChecker<Self> {
        self.operand(tags::OP)
    }

    /// Checks that the result and the operand have the same type, then checks that type
    pub fn all_same(self) -> PrimitiveTypeChecker<Self> {
        self.same(&[tags::RESULT, tags::OP])
    }
}

impl HasTypeCheckerCtx for PrimitiveOperandsChecker<'_, 3> {
    type Unwrapped = ThreePrimitiveTypes;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx
    }

    fn unwrap(self) -> Self::Unwrapped {
        let [(_, result), (_, op1), (_, op2)] = self.operands;
        ThreePrimitiveTypes {
            result: result.unwrap(),
            op1: op1.unwrap(),
            op2: op2.unwrap(),
        }
    }
}

impl HasTypeCheckerCtx for PrimitiveOperandsChecker<'_, 2> {
    type Unwrapped = TwoPrimitiveTypes;

    fn root_ctx(&mut self) -> &mut TypeCheckerCtx {
        self.ctx
    }

    fn unwrap(self) -> Self::Unwrapped {
        let [(_, result), (_, op)] = self.operands;
        TwoPrimitiveTypes {
            result: result.unwrap(),
            op: op.unwrap(),
        }
    }
}
