use std::mem::size_of;

use crate::opcodes::Opcode;
use crate::code::Chunk;

use super::DecodeResult;

pub(super) fn decode_u64_ld0(_: &Chunk) -> DecodeResult {
    DecodeResult::new(1, format!("{:?}", Opcode::U64Ld0))
}
pub(super) fn decode_i64_ld0(_: &Chunk) -> DecodeResult {
    DecodeResult::new(1, format!("{:?}", Opcode::I64Ld0))
}

pub(super) fn decode_ld_type0(chunk: &Chunk) -> DecodeResult {
    let type_ref = chunk.read_ref(0);
    let repr = format!("{:?} #{}", Opcode::LdTyped0, type_ref);
    DecodeResult::new(1 + refs(1), repr)
}

pub(super) fn decode_ld_type(chunk: &Chunk) -> DecodeResult {
    let type_ref = chunk.read_ref(0);
    let val_ref = chunk.read_ref(1);
    let repr = format!("{:?} #{} #{}", Opcode::LdType, type_ref, val_ref);
    DecodeResult::new(1 + refs(2), repr)
}

pub(super) fn decode_u_add(chunk: &Chunk) -> DecodeResult {
    decode_three_stack_ref(Opcode::UAdd, chunk)
}

pub(super) fn decode_i_add(chunk: &Chunk) -> DecodeResult {
    decode_three_stack_ref(Opcode::IAdd, chunk)
}

pub(super) fn decode_f_add(chunk: &Chunk) -> DecodeResult {
    decode_three_stack_ref(Opcode::FAdd, chunk)
}

fn decode_three_stack_ref(code: Opcode, chunk: &Chunk) -> DecodeResult {
    let res_ref = chunk.read_ref(0);
    let op1_ref = chunk.read_ref(1);
    let op2_ref = chunk.read_ref(2);
    let repr = format!(
        "{:?} @<res>{} @<op1>{} @<op2>{}",
        code, res_ref, op1_ref, op2_ref
    );
    DecodeResult::new(1 + refs(3), repr)
}

pub(crate) fn noop(_: &Chunk) -> DecodeResult {
    panic!("unknown opcode");
}

pub(super) fn decode_wide(chunk: &Chunk) -> DecodeResult {
    noop(chunk)
}

const fn refs(n_refs: usize) -> usize {
    n_refs * size_of::<usize>()
}
