use thiserror::Error;

use crate::types::RefKind;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct ValueLockData {
    pub lock_cycle: usize,
    pub partial_lock: bool,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ValueLock {
    None,
    Ref(ValueLockData),
    Mut(ValueLockData),
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
    #[error("Attempted to acquire a partial ref lock on a value, but value is already locked as fully as ref")]
    RefPartialLockButRefFullLock,
}

impl ValueLock {
    pub fn lock_cycle(&self) -> Option<usize> {
        match self {
            ValueLock::None => None,
            ValueLock::Ref(c) => Some(c.lock_cycle),
            ValueLock::Mut(c) => Some(c.lock_cycle),
        }
    }

    #[deprecated]
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
                *self = ValueLock::Mut(ValueLockData {
                    lock_cycle: current_cycle,
                    partial_lock: false,
                });
                Ok(())
            }
            ValueLock::Ref(_) => Err(LockError::MutLockButRefLocked),
            ValueLock::Mut(_) => Err(LockError::MutLockButMutLocked),
        }
    }

    pub fn add_ref_lock(&mut self, current_cycle: usize) -> Result<(), LockError> {
        match self {
            ValueLock::None => {
                *self = ValueLock::Ref(ValueLockData {
                    lock_cycle: current_cycle,
                    partial_lock: false,
                });
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

    pub fn add_lock_partial(
        &mut self,
        current_cycle: usize,
        ref_kind: RefKind,
    ) -> Result<(), LockError> {
        match ref_kind {
            RefKind::Mut => self.add_mut_lock_partial(current_cycle),
            RefKind::Ref => self.add_ref_lock_partial(current_cycle),
        }
    }

    pub fn add_mut_lock_partial(&mut self, current_cycle: usize) -> Result<(), LockError> {
        match self {
            ValueLock::None => {
                *self = ValueLock::Mut(ValueLockData {
                    lock_cycle: current_cycle,
                    partial_lock: true,
                });
                Ok(())
            }
            ValueLock::Mut(data) if data.partial_lock => Ok(()),
            ValueLock::Mut(_) => Err(LockError::MutLockButMutLocked),
            ValueLock::Ref(_) => Err(LockError::MutLockButRefLocked),
        }
    }

    pub fn add_ref_lock_partial(&mut self, current_cycle: usize) -> Result<(), LockError> {
        match self {
            ValueLock::None => {
                *self = ValueLock::Ref(ValueLockData {
                    lock_cycle: current_cycle,
                    partial_lock: true,
                });
                Ok(())
            }
            ValueLock::Ref(data) if data.partial_lock => Ok(()),
            ValueLock::Ref(_) => Err(LockError::RefPartialLockButRefFullLock),
            ValueLock::Mut(_) => Err(LockError::RefLockButMutLocked),
        }
    }
}
