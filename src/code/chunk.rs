use crate::code::RefSource;
use crate::opcodes::{Opcode, OpcodeKind};

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
    pub(crate) fn read_byte(&self, index: usize) -> Option<u8> {
        self.bytes.get(self.offset + index).copied()
    }

    #[allow(dead_code)]
    pub(crate) fn single_opcode(&self) -> Option<Opcode> {
        Opcode::single(self.read_byte(0)?)
    }

    #[allow(dead_code)]
    pub(crate) fn opcode(&self, kind: OpcodeKind) -> Option<Opcode> {
        Opcode::from_kind(self.read_byte(0)?, kind)
    }

    pub fn full_opcode(&self) -> Option<Opcode> {
        let first_byte = self.read_byte(0)?;
        if first_byte != u8::MAX {
            Opcode::single(first_byte)
        } else {
            let second_byte = self.read_byte(1)?;
            Opcode::double(second_byte)
        }
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
    fn read_from_offset(&self, offset: usize, size: usize) -> Option<&[u8]> {
        let offset = self.offset + offset;
        if offset + size <= self.bytes.len() {
            Some(&self.bytes[offset..offset + size])
        } else {
            None
        }
    }
}
