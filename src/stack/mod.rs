use std::ops::Range;

use crate::types::HasVmType;

pub mod data;

pub fn get_stack_range<T: HasVmType>(base: usize, t: &T) -> Range<usize> {
    let until = base + t.vm_type().size();
    base..until
}
