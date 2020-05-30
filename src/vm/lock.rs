use crate::types::RefKind;

use thiserror::Error;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ValueLock {
    None,
    Ref(usize),
    Mut(usize),
}

impl Default for ValueLock {
    fn default() -> Self {
        ValueLock::None
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum DerefLock {
    None,
    Ref,
    Mut,
}

impl Default for DerefLock {
    fn default() -> Self {
        DerefLock::None
    }
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
}

impl ValueLock {
    pub fn lock_cycle(&self) -> Option<usize> {
        match self {
            ValueLock::None => None,
            ValueLock::Ref(c) => Some(*c),
            ValueLock::Mut(c) => Some(*c),
        }
    }

    pub fn effective_lock(&self, current_cycle: usize) -> &ValueLock {
        match self.lock_cycle() {
            None => &ValueLock::None,
            Some(c) if current_cycle < c => &ValueLock::None,
            Some(_) => self,
        }
    }

    pub fn is_locked(&self) -> bool {
        !matches!(self, ValueLock::None)
    }

    pub fn can_be_ref_locked(&self) -> bool {
        match self {
            ValueLock::None => true,
            ValueLock::Ref(_) => true,
            ValueLock::Mut(_) => false,
        }
    }

    pub fn can_be_mut_locked(&self) -> bool {
        match self {
            ValueLock::None => true,
            ValueLock::Ref(_) => false,
            ValueLock::Mut(_) => false,
        }
    }

    pub fn can_be_locked(&self, ref_kind: RefKind) -> bool {
        match ref_kind {
            RefKind::Mut => self.can_be_mut_locked(),
            RefKind::Ref => self.can_be_ref_locked(),
        }
    }

    pub fn add_mut_lock(&mut self, current_cycle: usize) -> Result<(), LockError> {
        match self {
            ValueLock::None => {
                *self = ValueLock::Mut(current_cycle);
                Ok(())
            }
            ValueLock::Ref(_) => Err(LockError::MutLockButRefLocked),
            ValueLock::Mut(_) => Err(LockError::MutLockButMutLocked),
        }
    }

    pub fn add_ref_lock(&mut self, current_cycle: usize) -> Result<(), LockError> {
        match self {
            ValueLock::None => {
                *self = ValueLock::Ref(current_cycle);
                Ok(())
            }
            ValueLock::Ref(_) => Ok(()),
            ValueLock::Mut(_) => Err(LockError::RefLockButMutLocked),
        }
    }

    pub fn add_lock(&mut self, current_cycle: usize, ref_kind: RefKind) -> Result<(), LockError> {
        match ref_kind {
            RefKind::Mut => self.add_mut_lock(current_cycle),
            RefKind::Ref => self.add_ref_lock(current_cycle),
        }
    }
}
