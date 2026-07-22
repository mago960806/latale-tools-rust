use crate::spf::FINFO_SIZE;

const FINFO_STATE: [u32; 16] = [
    0x6170_7865,
    0x3320_646e,
    0x7962_2d32,
    0x6b20_6574,
    0x4810_c68d,
    0xdead_19a0,
    0xb7ef_3972,
    0x2152_e65f,
    0x4810_c68d,
    0xdead_19a0,
    0xb7ef_3972,
    0x2152_e65f,
    0x0000_0000,
    0x0000_003a,
    0xffff_ffc5,
    0x4810_c6b7,
];

const RESOURCE_CONSTANTS: [u32; 4] = [0x6e20_6843, 0x646e_6167, 0x2065_6854, 0x6b63_6150];

const RESOURCE_BASE: u64 = 0xdead_19a0_4810_c68d;
const VERSION_MIX_CONSTANT: u64 = 0x9e37_79b9_7f4a_7c15;
const MURMUR_MIX_CONSTANT: u64 = 0xc6a4_a793_5bd1_e995;

/// 对单条 FINFO 记录执行 ChaCha20 XOR。
///
/// 新版客户端会为每条记录重新使用同一个初始状态。
pub(crate) fn crypt_finfo(data: &mut [u8; FINFO_SIZE]) {
    chacha20_xor(data, FINFO_STATE);
}

/// 对单个资源执行 ChaCha20 XOR。
///
/// 加密和解密使用相同操作，每个资源都会根据自身位置重新派生状态。
pub(crate) fn crypt_resource(data: &mut [u8], offset: i32, size: i32, raw_version: u32) {
    chacha20_xor(data, derive_resource_state(offset, size, raw_version));
}

fn derive_resource_state(offset: i32, size: i32, raw_version: u32) -> [u32; 16] {
    let offset = offset as u32;
    let size = size as u32;

    let mut mixed = ((offset as u64) << 32) | size as u64;

    // 客户端先将原始版本按有符号 32 位扩展，再进行 64 位模运算。
    let version64 = (raw_version as i32 as i64) as u64;
    let version_mix = version64.wrapping_mul(VERSION_MIX_CONSTANT);

    mixed ^= (mixed >> 13).wrapping_mul(MURMUR_MIX_CONSTANT);

    let q0 = mixed ^ version_mix ^ RESOURCE_BASE;
    let q1 = version_mix.wrapping_add(RESOURCE_BASE) ^ (mixed >> 32);
    let q2 = (mixed.wrapping_shl(17) ^ RESOURCE_BASE).wrapping_add(version_mix);
    let q3 = RESOURCE_BASE.rotate_left(32) ^ mixed.wrapping_add(version_mix);

    [
        RESOURCE_CONSTANTS[0],
        RESOURCE_CONSTANTS[1],
        RESOURCE_CONSTANTS[2],
        RESOURCE_CONSTANTS[3],
        q0 as u32,
        (q0 >> 32) as u32,
        q1 as u32,
        (q1 >> 32) as u32,
        q2 as u32,
        (q2 >> 32) as u32,
        q3 as u32,
        (q3 >> 32) as u32,
        mixed as u32,
        offset,
        size ^ raw_version,
        ((RESOURCE_BASE >> 32) as u32).wrapping_add(offset),
    ]
}

fn chacha20_xor(data: &mut [u8], mut state: [u32; 16]) {
    for chunk in data.chunks_mut(64) {
        let key_stream = chacha20_block(state);
        for (byte, key) in chunk.iter_mut().zip(key_stream) {
            *byte ^= key;
        }
        state[12] = state[12].wrapping_add(1);
    }
}

fn chacha20_block(initial_state: [u32; 16]) -> [u8; 64] {
    let mut state = initial_state;

    for _ in 0..10 {
        quarter_round(&mut state, 0, 4, 8, 12);
        quarter_round(&mut state, 1, 5, 9, 13);
        quarter_round(&mut state, 2, 6, 10, 14);
        quarter_round(&mut state, 3, 7, 11, 15);

        quarter_round(&mut state, 0, 5, 10, 15);
        quarter_round(&mut state, 1, 6, 11, 12);
        quarter_round(&mut state, 2, 7, 8, 13);
        quarter_round(&mut state, 3, 4, 9, 14);
    }

    let mut output = [0u8; 64];
    for (i, word) in state.iter().enumerate() {
        let word = word.wrapping_add(initial_state[i]).to_le_bytes();
        output[i * 4..i * 4 + 4].copy_from_slice(&word);
    }
    output
}

fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);

    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finfo_keystream_matches_documented_value() {
        let mut actual = [0u8; FINFO_SIZE];
        crypt_finfo(&mut actual);

        let expected = [
            0x86, 0x1a, 0xa7, 0xd7, 0xf5, 0xed, 0xb2, 0xad, 0xc0, 0x8b, 0x64, 0xa4, 0xc9, 0xd7,
            0xde, 0x1f, 0x05, 0x4d, 0xa0, 0x5a, 0x48, 0x32, 0xe0, 0xdc, 0x0e, 0xe7, 0x6b, 0xe4,
            0x84, 0x52, 0x37, 0x8b, 0xee, 0x73, 0x4f, 0x74, 0x78, 0xb9, 0xde, 0x41, 0xa2, 0x1d,
            0x82, 0x30, 0x44, 0xe2, 0xee, 0x74, 0xa8, 0x8f, 0xd8, 0x2f, 0xc7, 0xdd, 0x79, 0xd3,
            0xb6, 0x18, 0x30, 0x14, 0x71, 0x9f, 0xce, 0xc7, 0x7a, 0x31, 0x65, 0x9a, 0x3a, 0xc1,
            0x64, 0x07, 0x82, 0xcd, 0x1a, 0x80, 0x2b, 0x97, 0x26, 0xae, 0x3a, 0x2c, 0x5f, 0xd5,
            0x34, 0xc5, 0x4b, 0xb5, 0x87, 0xae, 0x59, 0x4c, 0xea, 0xb7, 0x8e, 0x2f, 0xdd, 0x8a,
            0x22, 0xf8, 0x22, 0x1b, 0x78, 0x6c, 0x2d, 0x64, 0x5c, 0x5e, 0x96, 0x06, 0xdf, 0x9f,
            0xb0, 0xfb, 0xb5, 0xb1, 0x80, 0xf8, 0x00, 0x35, 0xd0, 0x1f, 0xce, 0xbb, 0x63, 0x40,
            0x28, 0x32, 0x65, 0xbf, 0x1f, 0x3f, 0x3e, 0x2f, 0xcf, 0x24, 0x15, 0xef, 0x5d, 0xd7,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn resource_state_matches_documented_rowid_example() {
        let actual = derive_resource_state(0, 14_794, 0xf8c3_6632);
        let expected = [
            0x6e20_6843,
            0x646e_6167,
            0x2065_6854,
            0x6b63_6150,
            0x763a_8cc8,
            0xf5ae_eefd,
            0x68a8_c734,
            0xcc54_6a6e,
            0x4eaa_60a7,
            0x7f32_fed2,
            0x1f60_73d9,
            0xfc5b_3eec,
            0x5bd1_d05f,
            0x0000_0000,
            0xf8c3_5ff8,
            0xdead_19a0,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn xor_is_reversible() {
        let mut data = b"encrypted SPF resource".to_vec();
        let original = data.clone();
        let size = data.len() as i32;

        crypt_resource(&mut data, 1234, size, 0xf8c3_6632);
        assert_ne!(data, original);
        crypt_resource(&mut data, 1234, size, 0xf8c3_6632);

        assert_eq!(data, original);
    }
}
