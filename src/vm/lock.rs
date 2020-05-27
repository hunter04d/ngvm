#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ValueLock {
    None,
    Ref,
    Mut,
}

impl ValueLock {
    pub fn locked(self) -> bool {
        !matches!(self, ValueLock::None)
    }
}
