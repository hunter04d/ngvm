use std::convert::{TryFrom, TryInto};
use std::mem::size_of;

use crate::decoder::{HANDLERS as D_HANDLERS, DecodedOpcode};
use crate::interpreter::HANDLERS as I_HANDLERS;
use crate::model;
use crate::opcodes::Opcode;
use crate::refs::{
    PoolRef,
    Ref,
    StackRef, ThreeStackRefs,
    TwoStackRefs};
use crate::types::Type::StackFrame;
use crate::Vm;

/// Byte-code of this machine
/// A wrapper around the raw bytes
pub struct Code(Vec<u8>);

/// Chunk of bytecode that is currently being interpreted.
///
/// Used in handlers as an abstraction over the raw bytes.
#[derive(Debug, Clone)]
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

    pub(crate) fn advance(&mut self, by: usize) {
        self.offset += by;
    }

    pub(crate) fn set_offset(&mut self, new_offset: usize) {
        self.offset = new_offset;
    }

    #[inline]
    pub(crate) fn single(&self) -> u8 {
        self.bytes[self.offset]
    }

    pub(crate) fn opcode(&self) -> Opcode {
        Opcode::try_from(self.bytes[self.offset] as u16).expect("Invalid opcode")
    }

    /// Reads a `Ref` from bytecode
    ///
    /// `index` is the index of the ref in the bytecode
    #[inline]
    pub(crate) fn read_ref(&self, index: usize) -> Option<Ref> {
        self.read_ref_from_offset(1 + index * size_of::<Ref>())
    }

    pub(crate) fn read_two(&self) -> Option<TwoStackRefs> {
        let result = StackRef(self.read_ref(0)?);
        let op = StackRef(self.read_ref(1)?);
        Some(TwoStackRefs { result, op })
    }

    pub(crate) fn read_three(&self) -> Option<ThreeStackRefs> {
        let result = StackRef(self.read_ref(0)?);
        let op1 = StackRef(self.read_ref(1)?);
        let op2 = StackRef(self.read_ref(2)?);
        Some(ThreeStackRefs { result, op1, op2 })
    }

    pub(crate) fn read_ref_pool(&self, index: usize) -> Option<PoolRef> {
        self.read_ref(index).map(|v| v.into())
    }

    pub(crate) fn read_ref_stack(&self, index: usize) -> Option<StackRef> {
        self.read_ref(index).map(|v| v.into())
    }

    pub(crate) fn read_ref_from_offset(&self, offset: usize) -> Option<Ref> {
        const S: usize = size_of::<Ref>();
        // full offset to read the Ref from
        let offset = self.offset + offset;
        let bytes: [u8; S] = self.bytes[offset..offset + S].try_into().ok()?;
        Some(Ref::from_le_bytes(bytes))
    }

    pub(crate) fn offset(&self) -> usize {
        self.offset
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

pub struct DecodeResult {
    pub opcodes: Vec<DecodedOpcode>,
    pub size: usize,
    pub is_full: bool,
}

impl Code {
    pub fn interpret(&self, vm: &mut Vm) {
        let mut chunk = Chunk::from_code(self);
        while vm.ip < chunk.bytes.len() {
            let op_fn = I_HANDLERS[chunk.single() as usize];
            let consumed = op_fn(&chunk, vm);
            if let Some(e) = consumed.error {
                eprint!("{:?}", e);
                break;
            };
            if consumed.consumed != 0 {
                // we consumed in a linear nature
                vm.ip += consumed.consumed;
            }
            chunk.set_offset(vm.ip);
        }
    }

    pub fn decode(&self) -> DecodeResult {
        let mut chunk = Chunk::from_code(self);
        let mut opcodes = Vec::new();
        while chunk.offset < self.0.len() {
            let op_fn = D_HANDLERS[chunk.single() as usize];
            let res_opt = op_fn(&chunk);
            match res_opt {
                None =>  {
                    return DecodeResult {
                        opcodes,
                        size: chunk.offset,
                        is_full: false,
                    };
                },
                Some(res) => {
                    chunk.advance(res.consumed);
                    opcodes.push(res);
                }
            }
        }
        DecodeResult {
            opcodes,
            size: chunk.offset,
            is_full: true
        }
    }
}

impl DecodeResult {
    pub fn print(&self, print_bytes: bool) {
        let mut offset = 0usize;
        let w = self.size.to_string().len();
        for op in &self.opcodes {
            if print_bytes {
                let mut bytes = op.op_code.bytes();
                bytes.extend_from_slice(&op.refs.bytes());
                let bytes = bytes.into_iter().map(|v| format!("{:02x}", v)).collect::<Vec<_>>().join("");
                println!("{:<w$} 0x{:<64} {:?} {}", offset, bytes, op.op_code, op.refs, w = w);
            } else {
                println!("{:<w$} {:?} {}", offset, op.op_code, op.refs, w = w);
            }
            offset += op.consumed;
        }
    }
}

