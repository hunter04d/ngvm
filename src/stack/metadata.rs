use crate::types::Type;

pub struct StackMetadata {
    pub value_type: Type,
    pub index: usize,
    // TODO: other meta fields
}

impl StackMetadata {
    pub fn new(value_type: Type, index: usize) -> Self {
        StackMetadata { value_type, index }
    }

    pub fn is_single(&self) -> bool {
        self.value_type.is_single()
    }
}
