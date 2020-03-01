use std::convert::TryInto;
use std::mem::size_of;

use crate::decoder::decode_result::{PoolRef, StackRef};
use crate::decoder::HANDLERS as D_HANDLERS;
use crate::interpreter::HANDLERS as I_HANDLERS;
use crate::model;
use crate::opcodes::Ref;
use crate::refs::{ThreeRefs, TwoRefs};
use crate::Vm;

/// Byte-code of this machine
pub struct Code(Vec<u8>);

/// Chunk of bytecode that is currently being interpreted.
///
/// Used in handlers as an abstraction over the raw bytes.
pub(crate) struct Chunk<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Chunk<'a> {
    #[allow(dead_code)]
    pub(crate) fn get_bytes(&self) -> &[u8] {
        &self.bytes[self.offset..]
    }

    pub(crate) fn from_code(code: &'a Code) -> Chunk<'a> {
        Self {
            bytes: &code.0,
            offset: 0,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn advance(&mut self, by: usize) {
        self.offset += by;
    }

    pub(crate) fn set_offset(&mut self, new_offset: usize) {
        self.offset = new_offset;
    }

    #[inline]
    pub(crate) fn opcode_value(&self) -> u8 {
        self.bytes[self.offset]
    }

    #[inline]
    pub(crate) fn read_ref(&self, index: usize) -> Option<Ref> {
        self.read_ref_from_offset(1 + index * size_of::<usize>())
    }

    pub(crate) fn read_two(&self) -> Option<TwoRefs> {
        let result = self.read_ref(0)?;
        let op = self.read_ref(1)?;
        Some(TwoRefs { result, op })
    }

    pub(crate) fn read_three(&self) -> Option<ThreeRefs> {
        let result = self.read_ref(0)?;
        let op1 = self.read_ref(1)?;
        let op2 = self.read_ref(2)?;
        Some(ThreeRefs { result, op1, op2 })
    }

    pub(crate) fn read_ref_pool(&self, index: usize) -> Option<PoolRef> {
        self.read_ref(index).map(|v| v.into())
    }

    pub(crate) fn read_ref_stack(&self, index: usize) -> Option<StackRef> {
        self.read_ref(index).map(|v| v.into())
    }

    pub(crate) fn read_ref_from_offset(&self, offset: usize) -> Option<Ref> {
        let bytes: [u8; size_of::<Ref>()] = self.bytes
            [self.offset + offset..self.offset + offset + size_of::<Ref>()]
            .try_into()
            .ok()?;
        Some(Ref::from_le_bytes(bytes))
    }
}

impl Code {
    pub fn from_slice(slice: &[u8]) -> Self {
        Code(slice.to_owned())
    }

    pub fn from_vec(vec: Vec<u8>) -> Self {
        Code(vec)
    }
}

impl From<Vec<model::Opcode>> for Code {
    fn from(opcodes: Vec<model::Opcode>) -> Self {
        Self(
            opcodes
                .into_iter()
                .map(|op| op.to_bytes())
                .flatten()
                .collect::<Vec<_>>(),
        )
    }
}

impl Code {
    pub fn interpret(&self, vm: &mut Vm) {
        let mut chunk = Chunk::from_code(self);
        while vm.ip < chunk.bytes.len() {
            let op_fn = I_HANDLERS[chunk.opcode_value() as usize];
            let consumed = op_fn(&chunk, vm);
            if consumed != 0 {
                // we consumed in a linear nature
                vm.ip += consumed;
            }
            chunk.set_offset(vm.ip);
        }
    }

    pub fn decode(&self) {
        let print_bytes = false;
        let mut chunk = Chunk::from_code(self);
        while chunk.offset < self.0.len() {
            let op_fn = D_HANDLERS[chunk.opcode_value() as usize];
            let res = op_fn(&chunk);
            if print_bytes {
                let opcode = &self.0[chunk.offset..chunk.offset + res.consumed];
                let opcode_repr = opcode
                    .iter()
                    .map(|b| format!("{:0x}", b))
                    .collect::<Vec<_>>()
                    .join("");
                println!("{:<4} 0x{:>} {:>16}", chunk.offset, opcode_repr, res.repr);
            } else {
                println!("{:<4} {:>8}", chunk.offset, res.repr);
            }
            chunk.offset += res.consumed;
        }
    }
}
