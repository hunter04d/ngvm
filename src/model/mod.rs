//! This module contains types that represent VM as high level object model.

use std::collections::HashMap;
use std::convert::TryInto;
use std::iter::FromIterator;
use std::mem::size_of;

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use Opcode::*;

use crate::code::refs::*;
use crate::opcodes::Opcode as Nc;

/// Vm opcode represented as Rust enum (size constraints be dammed)
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    LdSS(PoolRef),
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
    Ge(ThreeStackRefs),
    Gt(ThreeStackRefs),
    Le(ThreeStackRefs),
    Lt(ThreeStackRefs),
    Eq(ThreeStackRefs),
    Ne(ThreeStackRefs),
    J {
        label: usize,
    },
    JC {
        label: usize,
        cond: StackRef,
    },
    JOffset {
        offset: usize,
    },
    JCOffset {
        offset: usize,
        cond: StackRef,
    },
    Label(usize),
    StartScope,
    EndScope,
    Scope(Vec<Opcode>),
    TakeRef(StackRef),
    TakeMut(StackRef),
    StartDeref(StackRef),
    EndDeref,
    Mv(StackRef, StackRef),
    SArrCreate0(usize, PoolRef),
    SArrGet {
        arr_ref: StackRef,
        index: StackRef,
    },
    SArrMut {
        arr_mut: StackRef,
        index: StackRef,
    },
    TraceStackValue(StackRef),
}

type OpcodeBytes = SmallVec<[u8; 32]>;

#[derive(Default)]
pub struct ToBytesCtx {
    label_table: HashMap<usize, usize>,
    jump_patch_table: Vec<usize>,
    bytes: Vec<u8>,
}

impl ToBytesCtx {
    pub(crate) fn new() -> Self {
        Self {
            label_table: Default::default(),
            jump_patch_table: Default::default(),
            bytes: Vec::new(),
        }
    }

    pub fn with_reserved_space(ops: &[Opcode]) -> Self {
        let capacity: usize = ops.iter().map(Opcode::size_in_bytes).sum();
        Self {
            label_table: Default::default(),
            jump_patch_table: Default::default(),
            bytes: Vec::with_capacity(capacity + ops.len()),
        }
    }

    pub fn convert(mut self, ops: &[Opcode]) -> Option<Vec<u8>> {
        for op in ops {
            let extend = op.to_bytes(&mut self)?;
            self.bytes.extend(extend);
        }
        for to_patch in self.jump_patch_table {
            const S: usize = size_of::<usize>();
            let from = 1 + to_patch;
            let until = from + S;
            let value: [u8; S] = self.bytes[from..until].try_into().ok()?;
            let key = usize::from_le_bytes(value);
            let value = self.label_table.get(&key)?.to_le_bytes();
            let bytes = value.iter().copied();

            self.bytes.splice(from..until, bytes);
        }
        Some(self.bytes)
    }
}

impl Opcode {
    pub fn to_bytes(&self, ctx: &mut ToBytesCtx) -> Option<OpcodeBytes> {
        let b = match self {
            Ld0U64 => single(Nc::U64Ld0),
            Ld0I64 => single(Nc::I64Ld0),
            LdTyped0 { type_location } => with_one_ref(Nc::LdTyped0, type_location.0),
            LDType {
                type_location,
                value_location,
            } => with_refs(Nc::LdType, &[type_location.0, value_location.0]),
            LdSS(p) => with_one_ref(Nc::LdSS, p.0),
            LdUnit => single(Nc::LdUnit),
            UAdd(v) => with_three_stack_refs(Nc::UAdd, v),
            USub(v) => with_three_stack_refs(Nc::USub, v),
            UMul(v) => with_three_stack_refs(Nc::UMul, v),
            UDiv(v) => with_three_stack_refs(Nc::UDiv, v),
            URem(v) => with_three_stack_refs(Nc::URem, v),
            IAdd(v) => with_three_stack_refs(Nc::IAdd, v),
            ISub(v) => with_three_stack_refs(Nc::ISub, v),
            IMul(v) => with_three_stack_refs(Nc::IMul, v),
            IDiv(v) => with_three_stack_refs(Nc::IDiv, v),
            IRem(v) => with_three_stack_refs(Nc::IRem, v),
            INeg(v) => with_two_stack_refs(Nc::INeg, v),
            FAdd(v) => with_three_stack_refs(Nc::FAdd, v),
            FSub(v) => with_three_stack_refs(Nc::FSub, v),
            FMul(v) => with_three_stack_refs(Nc::FMul, v),
            FDiv(v) => with_three_stack_refs(Nc::FDiv, v),
            FRem(v) => with_three_stack_refs(Nc::FRem, v),
            FNeg(v) => with_two_stack_refs(Nc::FNeg, v),

            LdTrue => single(Nc::LdTrue),
            LdFalse => single(Nc::LdFalse),
            BAnd(v) => with_three_stack_refs(Nc::BAnd, v),
            BOr(v) => with_three_stack_refs(Nc::BOr, v),
            BNot(v) => with_two_stack_refs(Nc::BNot, v),
            BBe(v) => with_two_stack_refs(Nc::BBe, v),
            BXor(v) => with_three_stack_refs(Nc::BXor, v),
            LAnd(v) => with_three_stack_refs(Nc::LAnd, v),
            LOr(v) => with_three_stack_refs(Nc::LOr, v),
            LNot(v) => with_three_stack_refs(Nc::LNot, v),
            LXor(v) => with_three_stack_refs(Nc::LXor, v),
            Shl(v) => with_three_stack_refs(Nc::Shl, v),
            Shr(v) => with_three_stack_refs(Nc::Shr, v),
            RotL(v) => with_three_stack_refs(Nc::RotL, v),
            RotR(v) => with_three_stack_refs(Nc::RotR, v),

            Ge(v) => with_three_stack_refs(Nc::Ge, v),
            Gt(v) => with_three_stack_refs(Nc::Gt, v),
            Le(v) => with_three_stack_refs(Nc::Le, v),
            Lt(v) => with_three_stack_refs(Nc::Lt, v),
            Eq(v) => with_three_stack_refs(Nc::Eq, v),
            Ne(v) => with_three_stack_refs(Nc::Ne, v),
            J { label } => {
                let offset = ctx.label_table.get(label);
                if let Some(offset) = offset {
                    with_offset(Nc::J, *offset)
                } else {
                    let len = ctx.bytes.len();
                    ctx.jump_patch_table.push(len);
                    with_offset(Nc::J, *label)
                }
            }
            JC { label, cond } => {
                let offset = ctx.label_table.get(label);
                if let Some(offset) = offset {
                    with_offset_and_ref(Nc::JC, *offset, cond.0)
                } else {
                    let len = ctx.bytes.len();
                    ctx.jump_patch_table.push(len);
                    with_offset_and_ref(Nc::JC, *label, cond.0)
                }
            }
            JOffset { offset } => with_offset(Nc::J, *offset),
            JCOffset { offset, cond } => with_offset_and_ref(Nc::JC, *offset, cond.0),
            Label(l) => {
                let len = ctx.bytes.len();
                ctx.label_table.insert(*l, len);
                OpcodeBytes::new()
            }
            StartScope => single(Nc::StartScope),
            EndScope => single(Nc::EndScope),
            TakeRef(r) => with_one_ref(Nc::TakeRef, r.0),
            TakeMut(r) => with_one_ref(Nc::TakeMut, r.0),
            TraceStackValue(v) => with_one_ref(Nc::TraceStackValue, v.0),
            Scope(opcodes) => {
                let mut result = SmallVec::new();
                result.extend_from_slice(&single(Nc::StartScope));

                for op in opcodes {
                    result.extend_from_slice(&op.to_bytes(ctx)?);
                }
                result.extend_from_slice(&single(Nc::EndScope));
                result
            }
            StartDeref(r) => with_one_ref(Nc::StartDeref, r.0),
            EndDeref => single(Nc::EndDeref),
            Mv(r, o) => with_two_stack_refs(Nc::Mv, &TwoStackRefs { result: *r, op: *o }),
            SArrCreate0(len, r) => with_offset_and_ref(Nc::SArrCreate0, *len, r.0),
            SArrGet { arr_ref, index } => with_two_refs(Nc::SArrRef, arr_ref.0, index.0),
            SArrMut { arr_mut, index } => with_two_refs(Nc::SArrMut, arr_mut.0, index.0),
        };
        Some(b)
    }

    pub fn size_in_bytes(&self) -> usize {
        match self {
            Ld0U64 => 1,
            Ld0I64 => 1,
            LdTyped0 { .. } => 1 + refs_size(2),
            LDType { .. } => 1 + refs_size(2),
            LdUnit => 1,
            LdTrue => 1,
            LdFalse => 1,
            LdSS(_) => 1 + refs_size(1),
            UAdd(_) => 1 + refs_size(3),
            USub(_) => 1 + refs_size(1),
            UMul(_) => 1 + refs_size(1),
            UDiv(_) => 1 + refs_size(1),
            URem(_) => 1 + refs_size(1),
            IAdd(_) => 1 + refs_size(3),
            ISub(_) => 1 + refs_size(3),
            IMul(_) => 1 + refs_size(3),
            IDiv(_) => 1 + refs_size(3),
            IRem(_) => 1 + refs_size(3),
            INeg(_) => 1 + refs_size(2),
            FAdd(_) => 1 + refs_size(3),
            FSub(_) => 1 + refs_size(3),
            FMul(_) => 1 + refs_size(3),
            FDiv(_) => 1 + refs_size(3),
            FRem(_) => 1 + refs_size(3),
            FNeg(_) => 1 + refs_size(2),
            BAnd(_) => 1 + refs_size(3),
            BOr(_) => 1 + refs_size(3),
            BNot(_) => 1 + refs_size(2),
            BBe(_) => 1 + refs_size(2),
            BXor(_) => 1 + refs_size(3),
            LAnd(_) => 1 + refs_size(3),
            LOr(_) => 1 + refs_size(3),
            LNot(_) => 1 + refs_size(2),
            LXor(_) => 1 + refs_size(3),
            Shl(_) => 1 + refs_size(3),
            Shr(_) => 1 + refs_size(3),
            RotL(_) => 1 + refs_size(3),
            RotR(_) => 1 + refs_size(3),
            Ge(_) => 1 + refs_size(3),
            Gt(_) => 1 + refs_size(3),
            Le(_) => 1 + refs_size(3),
            Lt(_) => 1 + refs_size(3),
            Eq(_) => 1 + refs_size(3),
            Ne(_) => 1 + refs_size(3),
            J { .. } => 1 + refs_size(1),
            JC { .. } => 1 + refs_size(2),
            JOffset { .. } => 1 + refs_size(1),
            JCOffset { .. } => 1 + refs_size(1),
            Label(_) => 0,
            StartScope => 1,
            EndScope => 1,
            Scope(ops) => 2 + ops.iter().map(|o| o.size_in_bytes()).sum::<usize>(),
            TakeRef(_) => 1 + refs_size(1),
            TakeMut(_) => 1 + refs_size(1),
            StartDeref(_) => 1 + refs_size(1),
            EndDeref => 1,
            Mv(_, _) => 1 + refs_size(2),
            SArrCreate0(_, _) => 1 + refs_size(2),
            TraceStackValue(_) => 1 + refs_size(1),
            SArrGet { .. } => 1 + refs_size(2),
            SArrMut { .. } => 1 + refs_size(2),
        }
    }
}

#[inline]
fn single(code: Nc) -> OpcodeBytes {
    OpcodeBytes::from_iter(code.bytes())
}

fn with_refs(code: Nc, refs: &[usize]) -> OpcodeBytes {
    let mut res = OpcodeBytes::new();
    res.extend_from_slice(&code.bytes());
    for &reference in refs {
        res.extend_from_slice(&reference.to_le_bytes());
    }
    res
}

fn with_one_ref(code: Nc, r: Ref) -> OpcodeBytes {
    let mut res = OpcodeBytes::new();
    res.extend_from_slice(&code.bytes());
    res.extend_from_slice(&r.to_le_bytes());
    res
}

fn with_two_refs(code: Nc, r1: Ref, r2: Ref) -> OpcodeBytes {
    let mut res = OpcodeBytes::new();
    res.extend_from_slice(&code.bytes());
    res.extend_from_slice(&r1.to_le_bytes());
    res.extend_from_slice(&r2.to_le_bytes());
    res
}

fn with_two_stack_refs(code: Nc, refs: &TwoStackRefs) -> OpcodeBytes {
    let mut res = OpcodeBytes::new();
    res.extend_from_slice(&code.bytes());
    res.extend_from_slice(&refs.result.0.to_le_bytes());
    res.extend_from_slice(&refs.op.0.to_le_bytes());
    res
}

fn with_three_stack_refs(code: Nc, refs: &ThreeStackRefs) -> OpcodeBytes {
    let mut res = OpcodeBytes::new();
    res.extend(code.bytes());
    res.extend_from_slice(&refs.result.0.to_le_bytes());
    res.extend_from_slice(&refs.op1.0.to_le_bytes());
    res.extend_from_slice(&refs.op2.0.to_le_bytes());
    res
}

fn with_offset(code: Nc, offset: usize) -> OpcodeBytes {
    let mut res = OpcodeBytes::new();
    res.extend(code.bytes());
    res.extend_from_slice(&offset.to_le_bytes());
    res
}

fn with_offset_and_ref(code: Nc, offset: usize, r: Ref) -> OpcodeBytes {
    let mut res = OpcodeBytes::new();
    res.extend(code.bytes());
    res.extend_from_slice(&offset.to_le_bytes());
    res.extend_from_slice(&r.to_le_bytes());
    res
}
