use handlers::{alu::f_ops::*, alu::i_ops::*, alu::shifts::*, alu::u_ops::*, load::*, *};

use crate::code::Chunk;
use crate::error::VmError;
use crate::interpreter::handlers::alu::handle_b_not;
use crate::refs::{ThreeRefs, TwoRefs};
use crate::stack::metadata::StackMetadata;
use crate::Vm;

pub mod handlers;
pub mod stack_tracer;

/// All the functions than handle the specific opcode
pub(crate) static HANDLERS: [fn(&Chunk, &mut Vm) -> InterpreterResult; 256] = [
    // U64 LD 0
    handle_u64_ld0, // 0
    // I64 LD 0
    handle_i64_ld0,           // 1
    handle_ld_typed0,         // 2
    handle_ld_type,           // 3
    handle_ld_unit,           // 4
    handle_ld_true,           // 5
    handle_ld_false,          // 6
    noop,                     // 7
    noop,                     // 8
    noop,                     // 9
    handle_u_add,             // 10
    handle_u_sub,             // 11
    handle_u_mul,             // 12
    handle_u_div,             // 13
    handle_u_rem,             // 14
    handle_i_add,             // 15
    handle_i_sub,             // 16
    handle_i_mul,             // 17
    handle_i_div,             // 18
    handle_i_rem,             // 19
    handle_i_neg,             // 20
    handle_f_add,             // 21
    handle_f_sub,             // 22
    handle_f_mul,             // 23
    handle_f_div,             // 24
    handle_f_rem,             // 25
    handle_f_neg,             // 26
    noop,                     // 27
    noop,                     // 28
    handle_b_not,             // 29
    noop,                     // 30
    noop,                     // 31
    noop,                     // 32
    noop,                     // 33
    noop,                     // 34
    noop,                     // 35
    handle_shl,               // 36
    handle_shr,               // 37
    handle_rotl,              // 38
    handle_rotr,              // 39
    noop,                     // 40
    noop,                     // 41
    noop,                     // 42
    noop,                     // 43
    noop,                     // 44
    noop,                     // 45
    noop,                     // 46
    noop,                     // 47
    noop,                     // 48
    noop,                     // 49
    noop,                     // 50
    noop,                     // 51
    noop,                     // 52
    noop,                     // 53
    noop,                     // 54
    noop,                     // 55
    noop,                     // 56
    noop,                     // 57
    noop,                     // 58
    noop,                     // 59
    noop,                     // 60
    noop,                     // 61
    noop,                     // 62
    noop,                     // 63
    noop,                     // 64
    noop,                     // 65
    noop,                     // 66
    noop,                     // 67
    noop,                     // 68
    noop,                     // 69
    noop,                     // 70
    noop,                     // 71
    noop,                     // 72
    noop,                     // 73
    noop,                     // 74
    noop,                     // 75
    noop,                     // 76
    noop,                     // 77
    noop,                     // 78
    noop,                     // 79
    noop,                     // 80
    noop,                     // 81
    noop,                     // 82
    noop,                     // 83
    noop,                     // 84
    noop,                     // 85
    noop,                     // 86
    noop,                     // 87
    noop,                     // 88
    noop,                     // 89
    noop,                     // 90
    noop,                     // 91
    noop,                     // 92
    noop,                     // 93
    noop,                     // 94
    noop,                     // 95
    noop,                     // 96
    noop,                     // 97
    noop,                     // 98
    noop,                     // 99
    noop,                     // 100
    noop,                     // 101
    noop,                     // 102
    noop,                     // 103
    noop,                     // 104
    noop,                     // 105
    noop,                     // 106
    noop,                     // 107
    noop,                     // 108
    noop,                     // 109
    noop,                     // 110
    noop,                     // 111
    noop,                     // 112
    noop,                     // 113
    noop,                     // 114
    noop,                     // 115
    noop,                     // 116
    noop,                     // 117
    noop,                     // 118
    noop,                     // 119
    noop,                     // 120
    noop,                     // 121
    noop,                     // 122
    noop,                     // 123
    noop,                     // 124
    noop,                     // 125
    noop,                     // 126
    noop,                     // 127
    noop,                     // 128
    noop,                     // 129
    noop,                     // 130
    noop,                     // 131
    noop,                     // 132
    noop,                     // 133
    noop,                     // 134
    noop,                     // 135
    noop,                     // 136
    noop,                     // 137
    noop,                     // 138
    noop,                     // 139
    noop,                     // 140
    noop,                     // 141
    noop,                     // 142
    noop,                     // 143
    noop,                     // 144
    noop,                     // 145
    noop,                     // 146
    noop,                     // 147
    noop,                     // 148
    noop,                     // 149
    noop,                     // 150
    noop,                     // 151
    noop,                     // 152
    noop,                     // 153
    noop,                     // 154
    noop,                     // 155
    noop,                     // 156
    noop,                     // 157
    noop,                     // 158
    noop,                     // 159
    noop,                     // 160
    noop,                     // 161
    noop,                     // 162
    noop,                     // 163
    noop,                     // 164
    noop,                     // 165
    noop,                     // 166
    noop,                     // 167
    noop,                     // 168
    noop,                     // 169
    noop,                     // 170
    noop,                     // 171
    noop,                     // 172
    noop,                     // 173
    noop,                     // 174
    noop,                     // 175
    noop,                     // 176
    noop,                     // 177
    noop,                     // 178
    noop,                     // 179
    noop,                     // 180
    noop,                     // 181
    noop,                     // 182
    noop,                     // 183
    noop,                     // 184
    noop,                     // 185
    noop,                     // 186
    noop,                     // 187
    noop,                     // 188
    noop,                     // 189
    noop,                     // 190
    noop,                     // 191
    noop,                     // 192
    noop,                     // 193
    noop,                     // 194
    noop,                     // 195
    noop,                     // 196
    noop,                     // 197
    noop,                     // 198
    noop,                     // 199
    noop,                     // 200
    noop,                     // 201
    noop,                     // 202
    noop,                     // 203
    noop,                     // 204
    noop,                     // 205
    noop,                     // 206
    noop,                     // 207
    noop,                     // 208
    noop,                     // 209
    noop,                     // 210
    noop,                     // 211
    noop,                     // 212
    noop,                     // 213
    noop,                     // 214
    noop,                     // 215
    noop,                     // 216
    noop,                     // 217
    noop,                     // 218
    noop,                     // 219
    noop,                     // 220
    noop,                     // 221
    noop,                     // 222
    noop,                     // 223
    noop,                     // 224
    noop,                     // 225
    noop,                     // 226
    noop,                     // 227
    noop,                     // 228
    noop,                     // 229
    noop,                     // 230
    noop,                     // 231
    noop,                     // 232
    noop,                     // 233
    noop,                     // 234
    noop,                     // 235
    noop,                     // 236
    noop,                     // 237
    noop,                     // 238
    noop,                     // 239
    noop,                     // 240
    noop,                     // 241
    noop,                     // 242
    noop,                     // 243
    noop,                     // 244
    noop,                     // 245
    noop,                     // 246
    noop,                     // 247
    noop,                     // 248
    noop,                     // 249
    noop,                     // 250
    noop,                     // 251
    noop,                     // 252
    noop,                     // 253
    handle_trace_stack_value, // 254
    // Handle two-byte instruction
    handle_wide, // 255
];

#[derive(Debug)]
pub(crate) struct InterpreterResult {
    pub(crate) consumed: usize,
    pub(crate) error: Option<VmError>,
}

impl InterpreterResult {
    fn new(consumed: usize) -> Self {
        Self {
            consumed,
            error: None,
        }
    }

    #[allow(dead_code)]
    fn new_with_error(consumed: usize, error: VmError) -> Self {
        Self {
            consumed,
            error: Some(error),
        }
    }

    #[allow(dead_code)]
    fn with_error(self, error: VmError) -> Self {
        Self {
            consumed: self.consumed,
            error: Some(error),
        }
    }
    fn with_error_opt(self, error: Option<VmError>) -> Self {
        Self {
            consumed: self.consumed,
            error,
        }
    }
}

impl From<usize> for InterpreterResult {
    #[inline]
    fn from(consumed: usize) -> Self {
        Self::new(consumed)
    }
}

struct ThreeStackMetadata<'a> {
    result: &'a StackMetadata,
    op1: &'a StackMetadata,
    op2: &'a StackMetadata,
}

struct TwoStackMetadata<'a> {
    result: &'a StackMetadata,
    op: &'a StackMetadata,
}

fn three_stack_metadata<'a>(
    vm: &'a Vm,
    refs: &ThreeRefs,
) -> Result<ThreeStackMetadata<'a>, VmError> {
    let result = vm.stack_metadata(refs.result)?;
    let op1 = vm.stack_metadata(refs.op1)?;
    let op2 = vm.stack_metadata(refs.op2)?;
    Ok(ThreeStackMetadata { result, op1, op2 })
}

fn two_stack_metadata<'a>(vm: &'a Vm, refs: &TwoRefs) -> Result<TwoStackMetadata<'a>, VmError> {
    let result = vm.stack_metadata(refs.result)?;
    let op = vm.stack_metadata(refs.op)?;
    Ok(TwoStackMetadata { result, op })
}

fn run<T, E>(fun: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    fun()
}
