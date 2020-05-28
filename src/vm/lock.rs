use crate::types::RefKind;

use thiserror::Error;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ValueLock {
    None,
    Ref(usize),
    Mut(usize),
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

    pub fn if_effectively_locked(&self, current_cycle: usize) -> bool {
        !matches!(self.effective_lock(current_cycle), ValueLock::None)
    }

    pub fn can_be_ref_locked(&self, current_cycle: usize) -> bool {
        let effective = self.effective_lock(current_cycle);
        match effective {
            ValueLock::None => true,
            ValueLock::Ref(_) => true,
            ValueLock::Mut(_) => false,
        }
    }

    pub fn can_be_mut_locked(&self, current_cycle: usize) -> bool {
        let effective = self.effective_lock(current_cycle);
        match effective {
            ValueLock::None => true,
            ValueLock::Ref(_) => false,
            ValueLock::Mut(_) => false,
        }
    }

    pub fn can_be_locked(&self, current_cycle: usize, ref_kind: RefKind) -> bool {
        match ref_kind {
            RefKind::Mut => self.can_be_mut_locked(current_cycle),
            RefKind::Ref => self.can_be_ref_locked(current_cycle),
        }
    }

    pub fn add_lock(&mut self, current_cycle: usize, ref_kind: RefKind) -> Result<(), LockError> {
        let effective = self.effective_lock(current_cycle);
        match ref_kind {
            RefKind::Mut => match effective {
                ValueLock::None => {
                    *self = ValueLock::Mut(current_cycle);
                    Ok(())
                }
                ValueLock::Ref(_) => Err(LockError::MutLockButRefLocked),
                ValueLock::Mut(_) => Err(LockError::MutLockButMutLocked),
            },
            RefKind::Ref => match effective {
                ValueLock::None => {
                    *self = ValueLock::Ref(current_cycle);
                    Ok(())
                }
                ValueLock::Ref(_) => Ok(()),
                ValueLock::Mut(_) => Err(LockError::RefLockButMutLocked),
            },
        }
    }
}
