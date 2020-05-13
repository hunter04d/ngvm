use std::mem::size_of;

use crate::decoder::{HANDLERS as D_HANDLERS, DecodedOpcode};
use crate::interpreter::HANDLERS as I_HANDLERS;
use crate::model;
use crate::refs::{
    PoolRef,
    Ref,
    StackRef, ThreeStackRefs,
    TwoStackRefs};
use crate::Vm;

mod chunk;

pub use chunk::Chunk;

/// Byte-code of this machine
/// A wrapper around the raw bytes
#[derive(Debug)]
pub struct Code(Vec<u8>);

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
            let op_fn = I_HANDLERS[chunk.read_byte() as usize];
            let consumed = op_fn(&chunk, vm);
            match consumed {
                Err(e) => {
                    eprint!("{:?}", e);
                    break;
                }
                Ok(count) if count > 0 => {
                    // we consumed in a linear nature
                    vm.ip += count;
                    // NOTE: don't use advance, position might be different
                    chunk.set_offset(vm.ip);
                }
                _ => {}
            }
        }
    }

    pub fn decode(&self) -> DecodeResult {
        let mut chunk = Chunk::from_code(self);
        let mut opcodes = Vec::new();
        while chunk.offset < self.0.len() {
            let op_fn = D_HANDLERS[chunk.read_byte() as usize];
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

pub trait RefSource {
    fn read_ref_from_offset(&self, offset: usize) -> Option<Ref>;
    /// Reads a `Ref` from bytecode
    ///
    /// `index` is the index of the ref in the bytecode
    #[inline]
    fn read_ref(&self, index: usize) -> Option<Ref> {
        self.read_ref_from_offset(1 + index * size_of::<Ref>())
    }

    fn read_two(&self) -> Option<TwoStackRefs> {
        let result = StackRef(self.read_ref(0)?);
        let op = StackRef(self.read_ref(1)?);
        Some(TwoStackRefs { result, op })
    }

    fn read_three(&self) -> Option<ThreeStackRefs> {
        let result = StackRef(self.read_ref(0)?);
        let op1 = StackRef(self.read_ref(1)?);
        let op2 = StackRef(self.read_ref(2)?);
        Some(ThreeStackRefs { result, op1, op2 })
    }

    #[inline]
    fn read_ref_pool(&self, index: usize) -> Option<PoolRef> {
        self.read_ref(index).map(|v| v.into())
    }

    #[inline]
    fn read_ref_stack(&self, index: usize) -> Option<StackRef> {
        self.read_ref(index).map(|v| v.into())
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
