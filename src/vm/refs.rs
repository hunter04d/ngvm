use crate::vm::ValueLocation;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum LocatedRef {
    Stack(usize),
    Transient(ValueLocation),
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
            Ok(PoolRef(self.read_ref_vm(index)?))
        }

        fn read_ref_stack_vm(&self, index: usize) -> Result<StackRef, Self::VmError> {
            Ok(StackRef(self.read_ref_vm(index)?))
        }

        fn read_ref_stack_with_offset_vm(&self, index: usize) -> Result<StackRef, Self::VmError> {
            self.read_ref_with_offset_vm(index).map(StackRef)
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
