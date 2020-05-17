use crate::code::{Chunk, RefSource};
use crate::opcodes::Opcode;

use super::model::{DecoderRef, DecoderRefs};
use super::tags;
use super::DecodedOpcode;

pub(super) fn decode_u64_ld0(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::zero(Opcode::U64Ld0))
}
pub(super) fn decode_i64_ld0(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::zero(Opcode::I64Ld0))
}

pub(super) fn decode_ld_unit(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::zero(Opcode::LdUnit))
}

pub(super) fn decode_ld_type0(chunk: &Chunk) -> Option<DecodedOpcode> {
    let type_ref = chunk.read_ref_pool(0)?;
    let refs = DecoderRefs::One(DecoderRef::new(type_ref, tags::TYPE));
    Some(DecodedOpcode::new(Opcode::LdTyped0, refs))
}

pub(super) fn decode_ld_type(chunk: &Chunk) -> Option<DecodedOpcode> {
    let type_ref = chunk.read_ref_pool(0)?;
    let val_ref = chunk.read_ref_pool(1)?;
    let refs = DecoderRefs::Two(
        DecoderRef::new(type_ref, tags::TYPE),
        DecoderRef::new(val_ref, tags::VALUE),
    );
    Some(DecodedOpcode::new(Opcode::LdType, refs))
}

fn decode_three_stack_ref(code: Opcode, chunk: &Chunk) -> Option<DecodedOpcode> {
    let res_ref = chunk.read_ref_stack(0)?;
    let op1_ref = chunk.read_ref_stack(1)?;
    let op2_ref = chunk.read_ref_stack(2)?;
    let refs = DecoderRefs::Three(
        DecoderRef::new(res_ref, tags::RESULT),
        DecoderRef::new(op1_ref, tags::OP1),
        DecoderRef::new(op2_ref, tags::OP2),
    );
    Some(DecodedOpcode::new(code, refs))
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
    decode_f_rem => Opcode::FRem,
    // bool ops
    decode_b_and => Opcode::BAnd,
    decode_b_or => Opcode::BOr,
    decode_b_xor => Opcode::BXor,
    // logical ops
    decode_l_and => Opcode::LAnd,
    decode_l_or => Opcode::LOr,
    decode_l_xor => Opcode::LXor,
    // shifts
    decode_shl => Opcode::Shl,
    decode_shr => Opcode::Shr,
    decode_rotr => Opcode::RotR,
    decode_rotl => Opcode::RotL,
    // comparisons
    decode_ge => Opcode::Ge,
    decode_gt => Opcode::Gt,
    decode_le => Opcode::Le,
    decode_lt => Opcode::Lt,
    decode_eq => Opcode::Eq,
    decode_ne => Opcode::Ne
}

fn decode_two_stack_ref(code: Opcode, chunk: &Chunk) -> Option<DecodedOpcode> {
    let res_ref = chunk.read_ref_stack(0)?;
    let op_ref = chunk.read_ref_stack(1)?;
    let refs = DecoderRefs::Two(
        DecoderRef::new(res_ref, tags::RESULT),
        DecoderRef::new(op_ref, tags::OP),
    );
    Some(DecodedOpcode::new(code, refs))
}

macro_rules! generate_two_decode {
    ($($fn_name: ident => $opcode: expr),*) => {
        $(
        pub(in crate::decoder) fn $fn_name(chunk: &Chunk) ->  Option<DecodedOpcode> {
            decode_two_stack_ref($opcode, chunk)
        })*

    };
}

generate_two_decode! {
    decode_i_neg => Opcode::INeg,
    decode_f_neg => Opcode::FNeg,

    decode_b_not => Opcode::BNot,
    decode_b_be => Opcode::BBe,

    decode_l_not => Opcode::LNot
}

pub(super) fn decode_ld_true(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::new(Opcode::LdTrue, DecoderRefs::Zero))
}

pub(super) fn decode_ld_false(_: &Chunk) -> Option<DecodedOpcode> {
    Some(DecodedOpcode::new(Opcode::LdFalse, DecoderRefs::Zero))
}

pub(super) fn decode_j(chunk: &Chunk) -> Option<DecodedOpcode> {
    let offset = chunk.read_ref_stack(0)?;
    let refs = DecoderRefs::One(DecoderRef::new(offset, tags::OFFSET));
    Some(DecodedOpcode::new(Opcode::J, refs))
}

pub(super) fn decode_jc(chunk: &Chunk) -> Option<DecodedOpcode> {
    let offset = chunk.read_ref(0)?;
    let condition = chunk.read_ref_stack(1)?;
    let refs = DecoderRefs::Two(
        DecoderRef::offset(offset, tags::OFFSET),
        DecoderRef::new(condition, tags::CONDITION));
    Some(DecodedOpcode::new(Opcode::J, refs))
}

pub(crate) fn noop(_: &Chunk) -> Option<DecodedOpcode> {
    panic!("unknown opcode");
}

pub(super) fn decode_debug_stack_value(chunk: &Chunk) -> Option<DecodedOpcode> {
    let stack_ref = chunk.read_ref_stack(0)?;
    let refs = DecoderRefs::One(DecoderRef::new(stack_ref, "v"));
    Some(DecodedOpcode::new(Opcode::TraceStackValue, refs))
}

pub(super) fn decode_wide(chunk: &Chunk) -> Option<DecodedOpcode> {
    noop(chunk)
}
