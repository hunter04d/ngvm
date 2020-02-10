//! This module contains types that represent VM as high level object model.

use std::mem::size_of;

use Opcode::*;

use crate::model::refs::ThreeRefs;
use crate::opcodes::Opcode as Nc;

pub mod refs;

/// Vm opcode represented as Rust enum (size constraints be dammed)
#[derive(Debug)]
pub enum Opcode {
    /// Load 0 u64
    U64Ld0,

    /// Load 0 i64
    I64Ld0,

    /// Load primitive types **empty** value from the constant pool
    LDTyped0 {
        type_location: usize,
    },

    /// Load type with specific value from constant pool
    LDType {
        type_location: usize,
        value_location: usize,
    },
    LdUnit,

    UAdd(ThreeRefs),
    USub(ThreeRefs),
    UMul(ThreeRefs),
    UDiv(ThreeRefs),
    URem(ThreeRefs),

    IAdd(ThreeRefs),
    ISub(ThreeRefs),
    IMul(ThreeRefs),
    IDiv(ThreeRefs),
    IRem(ThreeRefs),
    INeg(ThreeRefs),

    FAdd(ThreeRefs),
    FSub(ThreeRefs),
    FMul(ThreeRefs),
    FDiv(ThreeRefs),
    FRem(ThreeRefs),
    FNeg(ThreeRefs),

    /// Load static string from constant pool
    LdSS {
        location: usize,
    },

    TraceStackValue(usize),
    /// LOAD value from constant pool as **dynamic** string
    LDDS {
        location: usize,
    },
}

impl Opcode {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            U64Ld0 => vec![Nc::U64Ld0.into()],
            I64Ld0 => vec![Nc::I64Ld0.into()],
            LDTyped0 { type_location } => with_refs(Nc::LdTyped0, &[*type_location]),
            LDType {
                type_location,
                value_location,
            } => with_refs(Nc::LdType, &[*type_location, *value_location]),
            LdSS { location } => with_one_ref(Nc::LdSS, *location),
            LDDS { location } => with_one_ref(Nc::LdDS, *location),
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
            INeg(v) => with_three_refs(Nc::INeg, v),
            FAdd(v) => with_three_refs(Nc::FAdd, v),
            FSub(v) => with_three_refs(Nc::FSub, v),
            FMul(v) => with_three_refs(Nc::FMul, v),
            FDiv(v) => with_three_refs(Nc::FDiv, v),
            FRem(v) => with_three_refs(Nc::FRem, v),
            FNeg(v) => with_three_refs(Nc::FNeg, v),
            TraceStackValue(v) => with_one_ref(Nc::TraceStackValue, *v),
        }
    }
}

fn single(code: Nc) -> Vec<u8> {
    vec![u8::from(code)]
}

fn with_refs(code: Nc, refs: &[usize]) -> Vec<u8> {
    let mut res = Vec::with_capacity(1 + refs.len() * size_of::<usize>());
    res.push(code.into());
    for &reference in refs {
        res.extend_from_slice(&reference.to_le_bytes());
    }
    res
}

fn with_one_ref(code: Nc, r: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(1 + size_of::<usize>());
    res.push(code.into());
    res.extend_from_slice(&r.to_le_bytes());
    res
}

fn with_three_refs(code: Nc, refs: &ThreeRefs) -> Vec<u8> {
    let mut res = Vec::with_capacity(1 + 3 * size_of::<usize>());
    res.push(code.into());
    res.extend_from_slice(&refs.result.to_le_bytes());
    res.extend_from_slice(&refs.op1.to_le_bytes());
    res.extend_from_slice(&refs.op2.to_le_bytes());
    res
}
