//! Checks of the operands of an instruction
//!
//! Each operand is checked by a closure, the type it narrows the operand to is kept until the end
//! of the check. Operands checked to have the same type are merged into one, so a 3 operand check
//! continues as a 2 operand one

use crate::types::{PrimitiveType, ThreePrimitiveTypes, TwoPrimitiveTypes, VmType};

use super::primitive::PrimitiveTypeChecker;
use super::{Checked, HasTypeCheckerCtx, Tag, Taggable, TypeCheckerCtx, TypeError, tags};

/// The checker an operand check closure is given and returns
type OperandChecker<'c, T> = PrimitiveTypeChecker<&'c mut TypeCheckerCtx, T>;

/// Names of the instruction operands each operand of a check stands for
type Names = &'static [&'static str];

fn primitive(ctx: &mut TypeCheckerCtx, tag: &'static str, t: &VmType) -> Checked<PrimitiveType> {
    match t.primitive() {
        Some(p) => Checked::known(p),
        None => Checked::poisoned(ctx, TypeError::NotPrimitive(t.tag(tag))),
    }
}

/// Runs `check` on the operand of type `t`, named by `names`
fn check_operand<'c, P, T>(
    names: Names,
    t: Checked<P>,
    ctx: &'c mut TypeCheckerCtx,
    check: impl FnOnce(OperandChecker<'c, P>) -> OperandChecker<'c, T>,
) -> (Checked<T>, &'c mut TypeCheckerCtx) {
    let checked = check(PrimitiveTypeChecker {
        tag: Tag::Group(names),
        t,
        ctx,
    });
    (checked.t, checked.ctx)
}

/// Reports the operands if their known types are not all the same
fn report_not_same<const N: usize>(
    ctx: &mut TypeCheckerCtx,
    operands: [(Names, Checked<PrimitiveType>); N],
) {
    let known = || {
        operands
            .iter()
            .filter_map(|&(names, p)| p.value().map(|p| (names, p)))
    };
    let mut types = known().map(|(_, p)| p);
    if let Some(first) = types.next()
        && types.any(|p| p != first)
    {
        let tagged = known()
            .map(|(names, p)| VmType::Primitive(p).tag(Tag::Group(names)))
            .collect();
        ctx.report(TypeError::AllNotEqual(tagged));
    }
}

/// Checks the operands of a 3 operand instruction, all of them must be primitives
///
/// The type parameters are the types the operands are narrowed to
pub struct ThreeOperandsChecker<'c, R = PrimitiveType, O1 = PrimitiveType, O2 = PrimitiveType> {
    result: Checked<R>,
    op1: Checked<O1>,
    op2: Checked<O2>,
    ctx: &'c mut TypeCheckerCtx,
}

impl<'c> ThreeOperandsChecker<'c> {
    /// Starts the check, reporting the operands that are not primitives
    pub fn new(result: &VmType, op1: &VmType, op2: &VmType, ctx: &'c mut TypeCheckerCtx) -> Self {
        Self {
            result: primitive(ctx, tags::RESULT, result),
            op1: primitive(ctx, tags::OP1, op1),
            op2: primitive(ctx, tags::OP2, op2),
            ctx,
        }
    }
}

impl<'c, R, O1, O2> ThreeOperandsChecker<'c, R, O1, O2>
where
    R: Copy + Into<PrimitiveType>,
    O1: Copy + Into<PrimitiveType>,
    O2: Copy + Into<PrimitiveType>,
{
    pub fn result<T>(
        self,
        check: impl FnOnce(OperandChecker<'c, R>) -> OperandChecker<'c, T>,
    ) -> ThreeOperandsChecker<'c, T, O1, O2> {
        let (result, ctx) = check_operand(&[tags::RESULT], self.result, self.ctx, check);
        ThreeOperandsChecker {
            result,
            op1: self.op1,
            op2: self.op2,
            ctx,
        }
    }

    pub fn op1<T>(
        self,
        check: impl FnOnce(OperandChecker<'c, O1>) -> OperandChecker<'c, T>,
    ) -> ThreeOperandsChecker<'c, R, T, O2> {
        let (op1, ctx) = check_operand(&[tags::OP1], self.op1, self.ctx, check);
        ThreeOperandsChecker {
            result: self.result,
            op1,
            op2: self.op2,
            ctx,
        }
    }

    pub fn op2<T>(
        self,
        check: impl FnOnce(OperandChecker<'c, O2>) -> OperandChecker<'c, T>,
    ) -> ThreeOperandsChecker<'c, R, O1, T> {
        let (op2, ctx) = check_operand(&[tags::OP2], self.op2, self.ctx, check);
        ThreeOperandsChecker {
            result: self.result,
            op1: self.op1,
            op2,
            ctx,
        }
    }

    /// Checks that both operands have the same type, the check continues with the result and that
    /// type as the operand
    pub fn same_operands(self) -> TwoOperandsChecker<'c, R> {
        report_not_same(
            self.ctx,
            [
                (&[tags::OP1], self.op1.map(Into::into)),
                (&[tags::OP2], self.op2.map(Into::into)),
            ],
        );
        TwoOperandsChecker {
            result: self.result,
            op: self.op1.map(Into::into),
            names: &SAME_OPERANDS,
            ctx: self.ctx,
        }
    }

    /// Checks that the result and the first operand have the same type, the check continues with
    /// that type as the result and the second operand as the operand
    pub fn same_result_and_op1(self) -> TwoOperandsChecker<'c, PrimitiveType, O2> {
        report_not_same(
            self.ctx,
            [
                (&[tags::RESULT], self.result.map(Into::into)),
                (&[tags::OP1], self.op1.map(Into::into)),
            ],
        );
        TwoOperandsChecker {
            result: self.result.map(Into::into),
            op: self.op2,
            names: &SAME_RESULT_AND_OP1,
            ctx: self.ctx,
        }
    }

    /// Checks that the result and both operands have the same type, the returned checker checks
    /// that type
    pub fn all_same(self) -> PrimitiveTypeChecker<Self> {
        let result = self.result.map(Into::into);
        report_not_same(
            self.ctx,
            [
                (&[tags::RESULT], result),
                (&[tags::OP1], self.op1.map(Into::into)),
                (&[tags::OP2], self.op2.map(Into::into)),
            ],
        );
        PrimitiveTypeChecker {
            tag: Tag::Group(&[tags::RESULT, tags::OP1, tags::OP2]),
            t: result,
            ctx: self,
        }
    }

    pub fn parent(self) -> &'c mut TypeCheckerCtx {
        self.ctx
    }
}

impl<R, O1, O2> HasTypeCheckerCtx for ThreeOperandsChecker<'_, R, O1, O2> {
    type Unwrapped = ThreePrimitiveTypes<R, O1, O2>;

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

/// Names of the instruction operands the operands of a [`TwoOperandsChecker`] stand for
struct TwoNames {
    result: Names,
    op: Names,
    all: Names,
}

const TWO: TwoNames = TwoNames {
    result: &[tags::RESULT],
    op: &[tags::OP],
    all: &[tags::RESULT, tags::OP],
};

const SAME_OPERANDS: TwoNames = TwoNames {
    result: &[tags::RESULT],
    op: &[tags::OP1, tags::OP2],
    all: &[tags::RESULT, tags::OP1, tags::OP2],
};

const SAME_RESULT_AND_OP1: TwoNames = TwoNames {
    result: &[tags::RESULT, tags::OP1],
    op: &[tags::OP2],
    all: &[tags::RESULT, tags::OP1, tags::OP2],
};

/// Checks the operands of a 2 operand instruction, or of a 3 operand one where two of them have
/// the same type, all of them must be primitives
///
/// The type parameters are the types the operands are narrowed to
pub struct TwoOperandsChecker<'c, R = PrimitiveType, O = PrimitiveType> {
    result: Checked<R>,
    op: Checked<O>,
    names: &'static TwoNames,
    ctx: &'c mut TypeCheckerCtx,
}

impl<'c> TwoOperandsChecker<'c> {
    /// Starts the check, reporting the operands that are not primitives
    pub fn new(result: &VmType, op: &VmType, ctx: &'c mut TypeCheckerCtx) -> Self {
        Self {
            result: primitive(ctx, tags::RESULT, result),
            op: primitive(ctx, tags::OP, op),
            names: &TWO,
            ctx,
        }
    }
}

impl<'c, R, O> TwoOperandsChecker<'c, R, O>
where
    R: Copy + Into<PrimitiveType>,
    O: Copy + Into<PrimitiveType>,
{
    pub fn result<T>(
        self,
        check: impl FnOnce(OperandChecker<'c, R>) -> OperandChecker<'c, T>,
    ) -> TwoOperandsChecker<'c, T, O> {
        let (result, ctx) = check_operand(self.names.result, self.result, self.ctx, check);
        TwoOperandsChecker {
            result,
            op: self.op,
            names: self.names,
            ctx,
        }
    }

    pub fn op<T>(
        self,
        check: impl FnOnce(OperandChecker<'c, O>) -> OperandChecker<'c, T>,
    ) -> TwoOperandsChecker<'c, R, T> {
        let (op, ctx) = check_operand(self.names.op, self.op, self.ctx, check);
        TwoOperandsChecker {
            result: self.result,
            op,
            names: self.names,
            ctx,
        }
    }

    /// Checks that the result and the operand have the same type, the returned checker checks that
    /// type
    pub fn all_same(self) -> PrimitiveTypeChecker<Self> {
        let result = self.result.map(Into::into);
        report_not_same(
            self.ctx,
            [
                (self.names.result, result),
                (self.names.op, self.op.map(Into::into)),
            ],
        );
        PrimitiveTypeChecker {
            tag: Tag::Group(self.names.all),
            t: result,
            ctx: self,
        }
    }

    pub fn parent(self) -> &'c mut TypeCheckerCtx {
        self.ctx
    }
}

impl<R, O> HasTypeCheckerCtx for TwoOperandsChecker<'_, R, O> {
    type Unwrapped = TwoPrimitiveTypes<R, O>;

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
