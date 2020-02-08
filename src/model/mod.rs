//! This module contains types that represent VM as high level object model.

use std::mem::size_of;

use Opcode::*;

use crate::opcodes::Opcode as Nc;

/// Vm opcode represented as Rust enum (size constraints be dammed)
#[derive(Debug)]
pub enum Opcode {
    /// Load 0 u64
    U64Ld0,

    /// Load 0 i64
    I64Ld0,

    /// Load primitive types **empty** value from the constant pool
    LDTyped0 { type_location: usize },

    /// Load type with specific value from constant pool
    LDType {
        type_location: usize,
        value_location: usize,
    },

    IAdd {
        result: usize,
        op1: usize,
        op2: usize,
    },
    UAdd {
        result: usize,
        op1: usize,
        op2: usize,
    },
    FAdd {
        result: usize,
        op1: usize,
        op2: usize,
    },

    /// Load static string from constant pool
    LdSS { location: usize },

    /// LOAD value from constant pool as **dynamic** string
    LDDS { location: usize },
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
            IAdd { result, op1, op2 } => with_refs(Nc::IAdd, &[*result, *op1, *op2]),
            UAdd { result, op1, op2 } => with_refs(Nc::UAdd, &[*result, *op1, *op2]),
            FAdd { result, op1, op2 } => with_refs(Nc::FAdd, &[*result, *op1, *op2]),
            LdSS { location: _ } => Vec::new(),
            LDDS { location: _ } => Vec::new(),
        }
    }
}

fn with_refs(code: Nc, refs: &[usize]) -> Vec<u8> {
    let mut res = Vec::with_capacity(1 + refs.len() * size_of::<usize>());
    res.push(code.into());
    for &reference in refs {
        res.extend_from_slice(&reference.to_le_bytes());
    }
    res
}
