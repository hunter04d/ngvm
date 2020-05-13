use std::convert::{TryFrom, TryInto};
use std::mem::size_of;

use crate::code::RefSource;
use crate::opcodes::Opcode;
use crate::refs::Ref;

use super::Code;

/// Chunk of bytecode that is currently being interpreted.
///
/// Used in handlers as an abstraction over the raw bytes.
#[derive(Debug, Clone)]
pub struct Chunk<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) offset: usize,
}

impl<'a> Chunk<'a> {
    #[allow(dead_code)]
    pub(crate) fn get_bytes(&self) -> &[u8] {
        &self.bytes[self.offset..]
    }

    pub fn from_code(code: &'a Code) -> Chunk<'a> {
        Self {
            bytes: &code.0,
            offset: 0,
        }
    }

    pub(crate) fn advance(&mut self, by: usize) {
        self.offset += by;
    }

    pub(crate) fn set_offset(&mut self, new_offset: usize) {
        self.offset = new_offset;
    }

    #[inline]
    pub(crate) fn read_byte(&self) -> u8 {
        self.bytes[self.offset]
    }

    pub(crate) fn single_opcode(&self) -> Opcode {
        Opcode::try_from(self.bytes[self.offset] as u16).expect("Invalid opcode")
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl<'a> From<&'a Code> for Chunk<'a> {
    fn from(code: &'a Code) -> Self {
        Self::from_code(code)
    }
}

impl<'a> RefSource for Chunk<'a> {
    fn read_ref_from_offset(&self, offset: usize) -> Option<Ref> {
        const S: usize = size_of::<Ref>();
        // full offset to read the Ref from
        let offset = self.offset + offset;
        let bytes: [u8; S] = self.bytes[offset..offset + S].try_into().ok()?;
        Some(Ref::from_le_bytes(bytes))
    }
}
