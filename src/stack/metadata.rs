use crate::refs::StackRef;
use crate::types::Type;

#[derive(Debug)]
pub struct StackMetadata {
    pub value_type: Type,
    pub index: StackRef,
    // TODO: other meta fields
}

impl StackMetadata {
    pub fn new(value_type: Type, index: StackRef) -> Self {
        Self { value_type, index }
    }

    pub fn is_single(&self) -> bool {
        self.value_type.is_single()
    }
}
