use functions::*;

use crate::code::Chunk;
use crate::opcodes::refs;

pub(crate) mod functions;

/// All the functions than handle the specific opcode
pub(crate) static HANDLERS: [fn(&Chunk) -> DecodeResult; 256] = [
    decode_u64_ld0,           // 0
    decode_i64_ld0,           // 1
    decode_ld_type0,          // 2
    decode_ld_type,           // 3
    decode_ld_unit,           // 4
    noop,                     // 5
    noop,                     // 6
    noop,                     // 7
    noop,                     // 8
    noop,                     // 9
    decode_u_add,             // 10
    decode_u_sub,             // 11
    decode_u_mul,             // 12
    decode_u_div,             // 13
    decode_u_rem,             // 14
    decode_i_add,             // 15
    decode_i_sub,             // 16
    decode_i_mul,             // 17
    decode_i_div,             // 18
    decode_i_rem,             // 19
    decode_i_neg,             // 20
    decode_f_add,             // 21
    decode_f_sub,             // 22
    decode_f_mul,             // 23
    decode_f_div,             // 24
    decode_f_rem,             // 25
    decode_f_neg,             // 26
    noop,                     // 27
    noop,                     // 28
    noop,                     // 29
    noop,                     // 30
    noop,                     // 31
    noop,                     // 32
    noop,                     // 33
    noop,                     // 34
    noop,                     // 35
    noop,                     // 36
    noop,                     // 37
    noop,                     // 38
    noop,                     // 39
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
    decode_debug_stack_value, // 254
    // Handle two-byte instruction
    decode_wide, // 255
];

/// The result of the decoding the input stream
pub(crate) struct DecodeResult {
    /// number of bytes consumed
    pub(crate) consumed: usize,
    /// String repr of the opcode
    pub(crate) repr: String,
}

impl DecodeResult {
    #[allow(dead_code)]
    pub(crate) fn new(consumed: usize, repr: String) -> Self {
        Self { consumed, repr }
    }

    pub(crate) fn with_refs(ref_count: usize, repr: String) -> Self {
        Self {
            consumed: 1 + refs(ref_count),
            repr,
        }
    }

    pub(crate) fn no_refs(repr: String) -> Self {
        Self { consumed: 0, repr }
    }
}
