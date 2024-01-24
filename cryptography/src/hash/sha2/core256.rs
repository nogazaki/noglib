//! Core hash computation algorithm for SHA-256 and SHA-224

use super::HasherCore;
use super::{SHA224_DIGEST_SIZE_BIT, SHA256_DIGEST_SIZE_BIT};
use crate::hash::DigestUser;
use crate::utils::{
    macros::{choose, majority},
    traits::BlockUser,
    types::BlockBuffer,
};

/// SHA-256 core block size in bits
const BLOCK_SIZE_BIT: usize = 512;
/// SHA-256 core block size in bytes
const BLOCK_SIZE_BYTE: usize = BLOCK_SIZE_BIT >> 3;

/// SHA-256 core digest size in bits
const DIGEST_SIZE_BIT: usize = SHA256_DIGEST_SIZE_BIT;
/// SHA-256 core digest size in bytes
const DIGEST_SIZE_BYTE: usize = DIGEST_SIZE_BIT >> 3;

/// SHA-256 core constants
const K_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98,
    0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8,
    0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819,
    0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
    0xc67178f2,
];

/// SHA-256 core logical function
#[inline(always)]
const fn big_sig_0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}
/// SHA-256 core logical function
#[inline(always)]
const fn big_sig_1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}
/// SHA-256 core logical function
#[inline(always)]
const fn small_sig_0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}
/// SHA-256 core logical function
#[inline(always)]
const fn small_sig_1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

/// SHA-256 core hash computation for a single block
fn sha256_core_digest_block(state: &mut [u32; 8], block: &[u8; BLOCK_SIZE_BYTE]) {
    let mut words = [0; 64];
    for (bytes, word) in block.chunks_exact(4).zip(words.iter_mut()) {
        *word = u32::from_be_bytes(bytes.try_into().unwrap_or_default());
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for t in 0..64 {
        if t >= 16 {
            words[t] = words[t - 7]
                .wrapping_add(words[t - 16])
                .wrapping_add(small_sig_1(words[t - 2]))
                .wrapping_add(small_sig_0(words[t - 15]));
        }

        let tmp1 = h
            .wrapping_add(big_sig_1(e))
            .wrapping_add(choose!(e, f, g))
            .wrapping_add(K_CONSTANTS[t])
            .wrapping_add(words[t]);
        let tmp2 = big_sig_0(a).wrapping_add(majority!(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(tmp1);
        d = c;
        c = b;
        b = a;
        a = tmp1.wrapping_add(tmp2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

/* -------------------------------------------------------------------------------- */

/// SHA-256 core object
#[derive(Debug)]
pub struct Sha256Core {
    /// Current state of this hashing instance
    state: [u32; DIGEST_SIZE_BYTE / 4],
    /// Temporary buffer, holding an incomplete block of data
    buffer: BlockBuffer<BLOCK_SIZE_BYTE>,
    /// Length of data processed
    msg_len: u64,
}

impl BlockUser for Sha256Core {
    const BLOCK_SIZE: usize = BLOCK_SIZE_BYTE;
}

impl DigestUser for Sha256Core {
    const DIGEST_SIZE: usize = DIGEST_SIZE_BYTE;
}

impl HasherCore for Sha256Core {
    fn new(truncated_digest_len_bit: usize) -> Self {
        let state = match truncated_digest_len_bit {
            SHA224_DIGEST_SIZE_BIT => [
                0xc1059ed8, 0x367cd507, 0x3070dd17, 0xf70e5939, 0xffc00b31, 0x68581511, 0x64f98fa7, 0xbefa4fa4,
            ],
            SHA256_DIGEST_SIZE_BIT => [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],

            _ => Default::default(),
        };

        Sha256Core {
            state,
            buffer: BlockBuffer::default(),
            msg_len: 0,
        }
    }

    fn compress(&mut self, data: &[u8]) {
        self.msg_len += data.len() as u64;
        self.buffer.process_data(data, |blocks| {
            for block in blocks {
                sha256_core_digest_block(&mut self.state, block);
            }
        });
    }

    fn finalize(&mut self) -> [u8; Self::DIGEST_SIZE] {
        #[allow(clippy::missing_docs_in_private_items)]
        const SUFFIX_POS: usize = Sha256Core::BLOCK_SIZE - core::mem::size_of::<u64>();

        let Self { state, buffer, msg_len } = self;

        let msg_len = *msg_len << 3;
        let pos = buffer.get_pos();
        let buffer = buffer.get_mut_buf();

        buffer[pos] = 0x80;
        buffer[pos + 1..].fill(0x00);

        if pos + 1 > SUFFIX_POS {
            sha256_core_digest_block(state, buffer);
            buffer.fill(0x00);
        }

        buffer[SUFFIX_POS..].copy_from_slice(&msg_len.to_be_bytes());
        sha256_core_digest_block(state, buffer);

        let mut digest = [0; DIGEST_SIZE_BYTE];
        for (block, integer) in digest.chunks_exact_mut(4).zip(self.state) {
            block.copy_from_slice(&integer.to_be_bytes());
        }

        digest
    }
}
