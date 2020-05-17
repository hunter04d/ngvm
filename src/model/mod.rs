//! This module contains types that represent VM as high level object model.

use std::mem::size_of;

use Opcode::*;

use crate::opcodes::Opcode as Nc;
use crate::refs::{PoolRef, Ref, StackRef, ThreeStackRefs, TwoStackRefs};

/// Vm opcode represented as Rust enum (size constraints be dammed)
#[derive(Debug)]
pub enum Opcode {
    /// Load 0 u64
    Ld0U64,

    /// Load 0 i64
    Ld0I64,

    /// Load primitive types **empty** value from the constant pool
    LdTyped0 {
        type_location: PoolRef,
    },

    /// Load type with specific value from constant pool
    LDType {
        type_location: PoolRef,
        value_location: PoolRef,
    },
    /// LD Unit
    LdUnit,
    // LD True
    LdTrue,
    LdFalse,

    /// Load static string from constant pool
    LdSS {
        location: PoolRef,
    },
    LdDS {
        location: PoolRef,
    },
    UAdd(ThreeStackRefs),
    USub(ThreeStackRefs),
    UMul(ThreeStackRefs),
    UDiv(ThreeStackRefs),
    URem(ThreeStackRefs),

    IAdd(ThreeStackRefs),
    ISub(ThreeStackRefs),
    IMul(ThreeStackRefs),
    IDiv(ThreeStackRefs),
    IRem(ThreeStackRefs),
    INeg(TwoStackRefs),

    FAdd(ThreeStackRefs),
    FSub(ThreeStackRefs),
    FMul(ThreeStackRefs),
    FDiv(ThreeStackRefs),
    FRem(ThreeStackRefs),
    FNeg(TwoStackRefs),

    BAnd(ThreeStackRefs),
    BOr(ThreeStackRefs),
    BNot(TwoStackRefs),
    BBe(TwoStackRefs),
    BXor(ThreeStackRefs),

    LAnd(ThreeStackRefs),
    LOr(ThreeStackRefs),
    LNot(ThreeStackRefs),
    LXor(ThreeStackRefs),

    Shl(ThreeStackRefs),
    Shr(ThreeStackRefs),
    RotL(ThreeStackRefs),
    RotR(ThreeStackRefs),

    TraceStackValue(StackRef),
}

impl Opcode {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Ld0U64 => single(Nc::U64Ld0),
            Ld0I64 => single(Nc::I64Ld0),
            LdTyped0 { type_location } => with_one_ref(Nc::LdTyped0, type_location.0),
            LDType {
                type_location,
                value_location,
            } => with_refs(Nc::LdType, &[type_location.0, value_location.0]),
            LdSS { location } => with_one_ref(Nc::LdSS, location.0),
            LdDS { location } => with_one_ref(Nc::LdDS, location.0),
            LdUnit => single(Nc::LdUnit),
            UAdd(v) => with_three_refs(Nc::UAdd, v),
            USub(v) => with_three_refs(Nc::USub, v),
            UMul(v) => with_three_refs(Nc::UMul, v),
            UDiv(v) => with_three_refs(Nc::UDiv, v),
            URem(v) => with_three_refs(Nc::URem, v),
            IAdd(v) => with_three_refs(Nc::IAdd, v),
            ISub(v) => with_three_refs(Nc::ISub, v),
            IMul(v) => with_three_refs(Nc::IMul, v),
            IDiv(v) => with_three_refs(Nc::IDiv, v),
            IRem(v) => with_three_refs(Nc::IRem, v),
            INeg(v) => with_two_refs(Nc::INeg, v),
            FAdd(v) => with_three_refs(Nc::FAdd, v),
            FSub(v) => with_three_refs(Nc::FSub, v),
            FMul(v) => with_three_refs(Nc::FMul, v),
            FDiv(v) => with_three_refs(Nc::FDiv, v),
            FRem(v) => with_three_refs(Nc::FRem, v),
            FNeg(v) => with_two_refs(Nc::FNeg, v),

            LdTrue => single(Nc::LdTrue),
            LdFalse => single(Nc::LdFalse),
            BAnd(v) => with_three_refs(Nc::BAnd, v),
            BOr(v) => with_three_refs(Nc::BOr, v),
            BNot(v) => with_two_refs(Nc::BNot, v),
            BBe(v) => with_two_refs(Nc::BBe, v),
            BXor(v) => with_three_refs(Nc::BXor, v),
            LAnd(v) => with_three_refs(Nc::LAnd, v),
            LOr(v) => with_three_refs(Nc::LOr, v),
            LNot(v) => with_three_refs(Nc::LNot, v),
            LXor(v) => with_three_refs(Nc::LXor, v),
            Shl(v) => with_three_refs(Nc::Shl, v),
            Shr(v) => with_three_refs(Nc::Shr, v),
            RotL(v) => with_three_refs(Nc::RotL, v),
            RotR(v) => with_three_refs(Nc::RotR, v),
            TraceStackValue(v) => with_one_ref(Nc::TraceStackValue, v.0),
        }
    }
}

#[inline]
fn single(code: Nc) -> Vec<u8> {
    code.bytes()
}

fn with_refs(code: Nc, refs: &[usize]) -> Vec<u8> {
    let mut res = Vec::with_capacity(1 + refs.len() * size_of::<usize>());
    res.extend_from_slice(&code.bytes());
    for &reference in refs {
        res.extend_from_slice(&reference.to_le_bytes());
    }
    res
}

fn with_one_ref(code: Nc, r: Ref) -> Vec<u8> {
    let mut res = Vec::with_capacity(code.size() + size_of::<usize>());
    res.extend_from_slice(&code.bytes());
    res.extend_from_slice(&r.to_le_bytes());
    res
}

fn with_two_refs(code: Nc, refs: &TwoStackRefs) -> Vec<u8> {
    let mut res = Vec::with_capacity(code.size() + 3 * size_of::<usize>());
    res.extend_from_slice(&code.bytes());
    res.extend_from_slice(&refs.result.0.to_le_bytes());
    res.extend_from_slice(&refs.op.0.to_le_bytes());
    res
}

fn with_three_refs(code: Nc, refs: &ThreeStackRefs) -> Vec<u8> {
    let mut res = Vec::with_capacity(code.size() + 3 * size_of::<usize>());
    res.extend_from_slice(&code.bytes());
    res.extend_from_slice(&refs.result.0.to_le_bytes());
    res.extend_from_slice(&refs.op1.0.to_le_bytes());
    res.extend_from_slice(&refs.op2.0.to_le_bytes());
    res
}
