use crate::code::Chunk;
use crate::opcodes::Opcode;

use super::DecodedOpcode;
use super::model::{DecoderRef, DecoderRefs};
use super::tags;

pub(super) fn decode_u64_ld0(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::single(Opcode::U64Ld0, DecoderRefs::Zero))
}
pub(super) fn decode_i64_ld0(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::single(Opcode::I64Ld0, DecoderRefs::Zero))
}

pub(super) fn decode_ld_unit(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::single(Opcode::LdUnit, DecoderRefs::Zero))
}

pub(super) fn decode_ld_type0(chunk: &Chunk) -> Option<DecodedOpcode> {
    let type_ref = chunk.read_ref_pool(0)?;
    let refs = DecoderRefs::One(
        DecoderRef::pool_with_tag(type_ref, tags::TYPE)
    );
    Some(DecodedOpcode::single(Opcode::LdTyped0, refs))
}

pub(super) fn decode_ld_type(chunk: &Chunk) -> Option<DecodedOpcode> {
    let type_ref = chunk.read_ref_pool(0).unwrap();
    let val_ref = chunk.read_ref_pool(1).unwrap();
    let refs = DecoderRefs::Two(
        DecoderRef::pool_with_tag(type_ref, tags::TYPE),
        DecoderRef::pool_with_tag(val_ref, tags::VALUE)
    );
    Some(DecodedOpcode::single(Opcode::LdType, refs))
}

fn decode_three_stack_ref(code: Opcode, chunk: &Chunk) -> Option<DecodedOpcode> {
    let res_ref = chunk.read_ref_stack(0)?;
    let op1_ref = chunk.read_ref_stack(1)?;
    let op2_ref = chunk.read_ref_stack(2)?;
    let refs = DecoderRefs::Three(
        DecoderRef::stack_with_tag(res_ref, tags::RESULT),
        DecoderRef::stack_with_tag(op1_ref, tags::OP1),
        DecoderRef::stack_with_tag(op2_ref, tags::OP2),
    );
    Some(DecodedOpcode::single(code, refs))
}

macro_rules! generate_three_decode {
    ($($fn_name: ident => $opcode: expr),*) => {
        $(
        pub(in crate::decoder) fn $fn_name(chunk: &Chunk) ->  Option<DecodedOpcode> {
            decode_three_stack_ref($opcode, chunk)
        })*

    };
}

generate_three_decode! {
    // u ops
    decode_u_add => Opcode::UAdd,
    decode_u_sub => Opcode::USub,
    decode_u_mul => Opcode::UMul,
    decode_u_div => Opcode::UDiv,
    decode_u_rem => Opcode::URem,
    // i ops
    decode_i_add => Opcode::IAdd,
    decode_i_sub => Opcode::ISub,
    decode_i_mul => Opcode::IMul,
    decode_i_div => Opcode::IDiv,
    decode_i_rem => Opcode::IRem,
    // f ops
    decode_f_add => Opcode::FAdd,
    decode_f_sub => Opcode::FSub,
    decode_f_mul => Opcode::FMul,
    decode_f_div => Opcode::FDiv,
    decode_f_rem => Opcode::FRem
}

pub(super) fn decode_i_neg(chunk: &Chunk) -> Option<DecodedOpcode> {
    let res_ref = chunk.read_ref_stack(0)?;
    let op_ref = chunk.read_ref_stack(0)?;
    let refs = DecoderRefs::Two(
        DecoderRef::stack_with_tag(res_ref, tags::RESULT),
        DecoderRef::stack_with_tag(op_ref, tags::OP)
    );
    Some(DecodedOpcode::single(Opcode::INeg, refs))
}


pub(super) fn decode_f_neg(chunk: &Chunk) -> Option<DecodedOpcode> {
    let res_ref = chunk.read_ref_stack(0)?;
    let op_ref = chunk.read_ref_stack(0)?;
    let refs = DecoderRefs::Two(
        DecoderRef::stack_with_tag(res_ref, tags::RESULT),
        DecoderRef::stack_with_tag(op_ref, tags::OP)
    );
    Some(DecodedOpcode::single(Opcode::FNeg, refs))
}

pub(super) fn decode_ld_true(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::single(Opcode::LdTrue, DecoderRefs::Zero))
}

pub(super) fn decode_ld_false(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::single(Opcode::LdFalse, DecoderRefs::Zero))
}

pub(crate) fn noop(_: &Chunk) -> Option<DecodedOpcode> {
    panic!("unknown opcode");
}

pub(super) fn decode_debug_stack_value(chunk: &Chunk) -> Option<DecodedOpcode> {
    let stack_ref = chunk.read_ref_stack(0)?;
    let refs = DecoderRefs::One(
        DecoderRef::stack(stack_ref)
    );
    Some(DecodedOpcode::single(Opcode::TraceStackValue, refs))
}

pub(super) fn decode_wide(chunk: &Chunk) -> Option<DecodedOpcode> {
    noop(chunk)
}
