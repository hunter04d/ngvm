use super::{HasTypeCheckerCtx, TypeError};

/// A type under check: either known or poisoned by a failed check.
///
/// A poisoned value can only be created by reporting an error to the root context, or derived
/// from another poisoned value, so a check that ends up with a poisoned type always fails
#[derive(Debug, Clone, Copy)]
pub struct Checked<T>(Option<T>);

impl<T> Checked<T> {
    pub fn known(value: T) -> Self {
        Self(Some(value))
    }

    /// Poisons the value, reporting `e` straight to the root context so that no intermediate
    /// context can swallow it
    pub fn poisoned(ctx: &mut impl HasTypeCheckerCtx, e: TypeError) -> Self {
        ctx.root_ctx().report(e);
        Self(None)
    }

    /// The value, `None` if it is poisoned
    pub fn value(self) -> Option<T> {
        self.0
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Checked<U> {
        Checked(self.0.map(f))
    }

    /// Continues the check of a known value, a poisoned value stays poisoned
    pub fn and_then<U>(self, f: impl FnOnce(T) -> Checked<U>) -> Checked<U> {
        Checked(self.0.and_then(|t| f(t).0))
    }

    /// Only to be called once the root context has no errors, which means nothing was poisoned
    pub(super) fn unwrap(self) -> T {
        self.0
            .expect("a poisoned type always has an error reported to the root context")
    }
}
