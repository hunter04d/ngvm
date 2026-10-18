use thiserror::Error;

use crate::types::RefKind;

/// Borrow state of a value
///
/// Every live reference holds exactly one borrow on the value it points to,
/// the borrow is released when that reference stops existing.
#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ValueLock {
    #[default]
    None,
    /// Borrowed by this many shared references
    Ref(usize),
    /// Borrowed by a single mutable reference, or being mutably dereferenced
    Mut,
    /// This array reference was used to borrow this many elements of the array
    Elements(usize),
}

#[repr(u8)]
#[derive(Debug, Default, Eq, PartialEq, Copy, Clone, Hash)]
pub enum DerefLock {
    #[default]
    None,
    Ref,
    Mut,
}

impl From<RefKind> for DerefLock {
    fn from(obj: RefKind) -> Self {
        match obj {
            RefKind::Ref => DerefLock::Ref,
            RefKind::Mut => DerefLock::Mut,
        }
    }
}

#[derive(Debug, Error)]
pub enum LockError {
    #[error("Attempted to acquire a mut lock on a value, but value is already locked as ref")]
    MutLockButRefLocked,
    #[error("Attempted to acquire a mut lock on a value, but value is already locked as mut")]
    MutLockButMutLocked,
    #[error("Attempted to acquire a ref lock on a value, but value is already locked as mut")]
    RefLockButMutLocked,
    #[error("Attempted to use a reference while elements borrowed through it are alive")]
    ElementsBorrowed,
    #[error("Attempted to borrow elements through a reference that is itself borrowed")]
    ElementsButLocked,
    #[error("Attempted to read a value that is mutably borrowed")]
    ReadButMutLocked,
    #[error("Attempted to overwrite or move a borrowed value")]
    WriteButLocked,
    #[error("Attempted to release a borrow that is not held")]
    NotLocked,
}

impl ValueLock {
    pub fn is_locked(&self) -> bool {
        !matches!(self, ValueLock::None)
    }

    /// Checks that the value can be read
    pub fn check_read(&self) -> Result<(), LockError> {
        match self {
            ValueLock::None | ValueLock::Ref(_) => Ok(()),
            ValueLock::Mut => Err(LockError::ReadButMutLocked),
            ValueLock::Elements(_) => Err(LockError::ElementsBorrowed),
        }
    }

    /// Checks that the value can be overwritten or moved out
    pub fn check_write(&self) -> Result<(), LockError> {
        match self {
            ValueLock::None => Ok(()),
            ValueLock::Elements(_) => Err(LockError::ElementsBorrowed),
            _ => Err(LockError::WriteButLocked),
        }
    }

    pub fn add_lock(&mut self, ref_kind: RefKind) -> Result<(), LockError> {
        *self = match (*self, ref_kind) {
            (ValueLock::None, RefKind::Ref) => ValueLock::Ref(1),
            (ValueLock::Ref(n), RefKind::Ref) => ValueLock::Ref(n + 1),
            (ValueLock::None, RefKind::Mut) => ValueLock::Mut,
            (ValueLock::Ref(_), RefKind::Mut) => return Err(LockError::MutLockButRefLocked),
            (ValueLock::Mut, RefKind::Mut) => return Err(LockError::MutLockButMutLocked),
            (ValueLock::Mut, RefKind::Ref) => return Err(LockError::RefLockButMutLocked),
            (ValueLock::Elements(_), _) => return Err(LockError::ElementsBorrowed),
        };
        Ok(())
    }

    pub fn release_lock(&mut self, ref_kind: RefKind) -> Result<(), LockError> {
        *self = match (*self, ref_kind) {
            (ValueLock::Ref(1), RefKind::Ref) => ValueLock::None,
            (ValueLock::Ref(n), RefKind::Ref) => ValueLock::Ref(n - 1),
            (ValueLock::Mut, RefKind::Mut) => ValueLock::None,
            _ => return Err(LockError::NotLocked),
        };
        Ok(())
    }

    /// Records that an element was borrowed through this (array reference) value
    pub fn add_element_lock(&mut self) -> Result<(), LockError> {
        *self = match *self {
            ValueLock::None => ValueLock::Elements(1),
            ValueLock::Elements(n) => ValueLock::Elements(n + 1),
            _ => return Err(LockError::ElementsButLocked),
        };
        Ok(())
    }

    pub fn release_element_lock(&mut self) -> Result<(), LockError> {
        *self = match *self {
            ValueLock::Elements(1) => ValueLock::None,
            ValueLock::Elements(n) => ValueLock::Elements(n - 1),
            _ => return Err(LockError::NotLocked),
        };
        Ok(())
    }
}
