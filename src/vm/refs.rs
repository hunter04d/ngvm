use num_traits::FromPrimitive;

use crate::error::VmError;
use crate::stack::data::{FromSingle, IntoStackData, StackData};
use crate::types::RefLocation;
use crate::vm::ValueLocation;

/// Stack data of a reference: the index (or address) of the value, then its [`RefLocation`]
pub type RefData = [StackData; 2];

/// The value a reference points to, which is what the stack data of a reference stores
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum LocatedRef {
    /// Absolute index of a stack value (not relative to the current stack frame)
    Stack(usize),
    Transient(ValueLocation),
}

impl LocatedRef {
    pub fn location(&self) -> RefLocation {
        match self {
            LocatedRef::Stack(_) => RefLocation::Stack,
            LocatedRef::Transient(ValueLocation::Stack(_)) => RefLocation::TransientOnStack,
            LocatedRef::Transient(ValueLocation::Heap(_)) => RefLocation::TransientOnHeap,
        }
    }

    /// Stack data of a reference that points to this location
    pub fn to_stack_data(self) -> RefData {
        let index = match self {
            LocatedRef::Stack(index) | LocatedRef::Transient(ValueLocation::Stack(index)) => index,
            LocatedRef::Transient(ValueLocation::Heap(ptr)) => ptr as usize,
        };
        [index.into_stack_data(), (self.location() as u8).into_stack_data()]
    }

    /// Reads the location from the stack data of a reference
    pub fn from_stack_data(data: &[StackData]) -> Result<Self, VmError> {
        let &[index, location] = data else {
            return Err(VmError::BadVmState);
        };
        let index = usize::from_single(index);
        let location = u8::try_from(u64::from_single(location))
            .ok()
            .and_then(RefLocation::from_u8);
        match location {
            Some(RefLocation::Stack) => Ok(LocatedRef::Stack(index)),
            Some(RefLocation::TransientOnStack) => {
                Ok(LocatedRef::Transient(ValueLocation::Stack(index)))
            }
            Some(RefLocation::TransientOnHeap) => {
                Ok(LocatedRef::Transient(ValueLocation::Heap(index as *const ())))
            }
            Some(RefLocation::Heap) | None => Err(VmError::BadVmState),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_located_ref_roundtrip() {
        let refs = [
            LocatedRef::Stack(0),
            LocatedRef::Stack(usize::MAX),
            LocatedRef::Transient(ValueLocation::Stack(7)),
            LocatedRef::Transient(ValueLocation::Heap(usize::MAX as *const ())),
        ];
        for r in refs {
            let data = r.to_stack_data();
            assert_eq!(LocatedRef::from_stack_data(&data).unwrap(), r);
        }
    }

    #[test]
    fn test_located_ref_rejects_bad_data() {
        let [index, _] = LocatedRef::Stack(1).to_stack_data();
        for location in [RefLocation::Heap as u64, 256 + RefLocation::Stack as u64, u64::MAX] {
            let data = [index, location.into_stack_data()];
            assert!(LocatedRef::from_stack_data(&data).is_err());
        }
        assert!(LocatedRef::from_stack_data(&[index]).is_err());
    }
}

pub(super) mod code {
    use crate::code::{refs::*, Chunk, RefSource};
    use crate::error::VmError;

    pub trait VmRefSource {
        type VmError: std::error::Error;

        fn read_from_offset_vm(&self, offset: usize, size: usize) -> Result<&[u8], Self::VmError>;

        fn read_ref_vm(&self, index: usize) -> Result<Ref, Self::VmError>;

        fn read_ref_with_offset_vm(&self, index: usize) -> Result<Ref, Self::VmError>;

        fn read_offset_vm(&self) -> Result<usize, Self::VmError>;

        fn read_two_vm(&self) -> Result<TwoStackRefs, Self::VmError> {
            let result = StackRef(self.read_ref_vm(0)?);
            let op = StackRef(self.read_ref_vm(1)?);
            Ok(TwoStackRefs { result, op })
        }

        fn read_three_vm(&self) -> Result<ThreeStackRefs, Self::VmError> {
            let result = StackRef(self.read_ref_vm(0)?);
            let op1 = StackRef(self.read_ref_vm(1)?);
            let op2 = StackRef(self.read_ref_vm(2)?);
            Ok(ThreeStackRefs { result, op1, op2 })
        }

        fn read_ref_pool_vm(&self, index: usize) -> Result<PoolRef, Self::VmError> {
            self.read_ref_vm(index).map(PoolRef)
        }

        fn read_ref_stack_vm(&self, index: usize) -> Result<StackRef, Self::VmError> {
            self.read_ref_vm(index).map(StackRef)
        }

        fn read_ref_stack_with_offset_vm(&self, index: usize) -> Result<StackRef, Self::VmError> {
            self.read_ref_with_offset_vm(index).map(StackRef)
        }
        fn read_ref_pool_with_offset_vm(&self, index: usize) -> Result<PoolRef, Self::VmError> {
            self.read_ref_with_offset_vm(index).map(PoolRef)
        }
    }

    impl VmRefSource for Chunk<'_> {
        type VmError = VmError;

        fn read_from_offset_vm(&self, offset: usize, size: usize) -> Result<&[u8], Self::VmError> {
            self.read_from_offset(offset, size)
                .ok_or(VmError::InvalidBytecode)
        }

        fn read_ref_vm(&self, index: usize) -> Result<Ref, Self::VmError> {
            self.read_ref(index).ok_or(VmError::InvalidBytecode)
        }

        fn read_ref_with_offset_vm(&self, index: usize) -> Result<Ref, Self::VmError> {
            self.read_ref_with_offset(index)
                .ok_or(VmError::InvalidBytecode)
        }

        fn read_offset_vm(&self) -> Result<usize, Self::VmError> {
            self.read_offset().ok_or(VmError::InvalidBytecode)
        }
    }
}
