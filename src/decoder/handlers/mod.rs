use crate::code::Chunk;
use crate::opcodes::Opcode;

use super::DecodeResult;

pub(super) fn decode_u64_ld0(_: &Chunk) -> DecodeResult {
    DecodeResult::from_no_refs(Opcode::U64Ld0)
}
pub(super) fn decode_i64_ld0(_: &Chunk) -> DecodeResult {
    DecodeResult::from_no_refs(Opcode::I64Ld0)
}

pub(super) fn decode_ld_unit(_: &Chunk) -> DecodeResult {
    DecodeResult::from_no_refs(Opcode::LdUnit)
}

pub(super) fn decode_ld_type0(chunk: &Chunk) -> DecodeResult {
    let type_ref = chunk.read_ref_pool(0);
    let repr = format!("{:?} {}", Opcode::LdTyped0, type_ref);
    DecodeResult::with_refs(1, repr)
}

pub(super) fn decode_ld_type(chunk: &Chunk) -> DecodeResult {
    let type_ref = chunk.read_ref_pool(0);
    let val_ref = chunk.read_ref_pool(1);
    let repr = format!("{:?} {} {}", Opcode::LdType, type_ref, val_ref);
    DecodeResult::with_refs(2, repr)
}

fn decode_three_stack_ref(code: Opcode, chunk: &Chunk) -> DecodeResult {
    let res_ref = chunk.read_ref_stack(0);
    let op1_ref = chunk.read_ref_stack(1);
    let op2_ref = chunk.read_ref_stack(2);
    let repr = format!(
        "{:?} <res>{} <op1>{} <op2>{}",
        code, res_ref, op1_ref, op2_ref
    );
    DecodeResult::with_refs(3, repr)
}

macro_rules! generate_three_decode {
    ($($fn_name: ident => $opcode: expr),*) => {
        $(
        pub(in crate::decoder) fn $fn_name(chunk: &Chunk) -> DecodeResult {
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

pub(super) fn decode_i_neg(chunk: &Chunk) -> DecodeResult {
    let stack_ref = chunk.read_ref_stack(0);
    let repr = format!("{:?} {}", Opcode::INeg, stack_ref);
    DecodeResult::with_refs(1, repr)
}

pub(super) fn decode_f_neg(chunk: &Chunk) -> DecodeResult {
    let stack_ref = chunk.read_ref_stack(0);
    let repr = format!("{:?} {}", Opcode::FNeg, stack_ref);
    DecodeResult::with_refs(1, repr)
}

pub(super) fn decode_ld_true(_: &Chunk) -> DecodeResult {
    DecodeResult::from_no_refs(Opcode::LdTrue)
}

pub(super) fn decode_ld_false(_: &Chunk) -> DecodeResult {
    DecodeResult::from_no_refs(Opcode::LdFalse)
}

pub(crate) fn noop(_: &Chunk) -> DecodeResult {
    panic!("unknown opcode");
}

pub(super) fn decode_debug_stack_value(chunk: &Chunk) -> DecodeResult {
    let stack_ref = chunk.read_ref_stack(0);
    let repr = format!("{:?} {}", Opcode::TraceStackValue, stack_ref);
    DecodeResult::with_refs(1, repr)
}

pub(super) fn decode_wide(chunk: &Chunk) -> DecodeResult {
    noop(chunk)
}
