//! Secure Hash Algorithm 1 ([SHA-1](https://en.wikipedia.org/wiki/SHA-1))

use super::hasher::{Hasher, HasherCore};
use crate::hash::DigestUser;
use crate::utils::{traits::BlockUser, types::BlockBuffer};

/// SHA-1 core block size in bits
const BLOCK_SIZE_BIT: usize = 512;
/// SHA-1 core block size in bytes
const BLOCK_SIZE_BYTE: usize = BLOCK_SIZE_BIT >> 3;

/// SHA-1 core digest size in bits
const DIGEST_SIZE_BIT: usize = 160;
/// SHA-1 core digest size in bytes
const DIGEST_SIZE_BYTE: usize = DIGEST_SIZE_BIT >> 3;

/// SHA-1 core constants
const K_CONSTANTS: [u32; 4] = [0x5a827999, 0x6ed9eba1, 0x8f1bbcdc, 0xca62c1d6];

/// SHA-1 core functions
macro_rules! sha1_functions {
    ($x:expr, $y:expr, $z:expr, $t:expr) => {
        match $t {
            0..=19 => $crate::utils::macros::choose!($x, $y, $z),
            20..=39 => $x ^ $y ^ $z,
            40..=59 => $crate::utils::macros::majority!($x, $y, $z),
            60..=79 => $x ^ $y ^ $z,

            _ => 0, // should be unreachable
        }
    };
}

/// SHA-1 core hash computation for a single block
fn sha1_core_digest_block(state: &mut [u32; 5], block: &[u8; BLOCK_SIZE_BYTE]) {
    let mut words = [0; 80];
    for (bytes, word) in block.chunks_exact(4).zip(words.iter_mut()) {
        *word = u32::from_be_bytes(bytes.try_into().unwrap_or_default());
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];

    for t in 0..80 {
        if t >= 16 {
            words[t] = (words[t - 3] ^ words[t - 8] ^ words[t - 14] ^ words[t - 16]).rotate_left(1);
        }

        let tmp = a
            .rotate_left(5)
            .wrapping_add(sha1_functions!(b, c, d, t))
            .wrapping_add(e)
            .wrapping_add(K_CONSTANTS[t / 20])
            .wrapping_add(words[t]);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = tmp;
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
}

/* -------------------------------------------------------------------------------- */

/// SHA-1 core object
#[derive(Debug)]
pub struct Sha1Core {
    /// Current state of this hashing instance
    state: [u32; DIGEST_SIZE_BYTE / 4],
    /// Temporary buffer, holding an incomplete block of data
    buffer: BlockBuffer<BLOCK_SIZE_BYTE>,
    /// Length of data processed
    msg_len: u64,
}

impl BlockUser for Sha1Core {
    const BLOCK_SIZE: usize = BLOCK_SIZE_BYTE;
}

impl DigestUser for Sha1Core {
    const DIGEST_SIZE: usize = DIGEST_SIZE_BYTE;
}

impl HasherCore for Sha1Core {
    fn new(_: usize) -> Self {
        Sha1Core {
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0],
            buffer: BlockBuffer::default(),
            msg_len: 0,
        }
    }

    fn compress(&mut self, data: &[u8]) {
        self.msg_len += data.len() as u64;
        self.buffer.process_data(data, |blocks| {
            for block in blocks {
                sha1_core_digest_block(&mut self.state, block);
            }
        });
    }

    fn finalize(&mut self) -> [u8; Self::DIGEST_SIZE] {
        #[allow(clippy::missing_docs_in_private_items)]
        const SUFFIX_POS: usize = Sha1Core::BLOCK_SIZE - core::mem::size_of::<u64>();

        let Self { state, buffer, msg_len } = self;

        let msg_len = *msg_len << 3;
        let pos = buffer.get_pos();
        let buffer = buffer.get_mut_buf();

        buffer[pos] = 0x80;
        buffer[pos + 1..].fill(0x00);

        if pos + 1 > SUFFIX_POS {
            sha1_core_digest_block(state, buffer);
            buffer.fill(0x00);
        }

        buffer[SUFFIX_POS..].copy_from_slice(&msg_len.to_be_bytes());
        sha1_core_digest_block(state, buffer);

        let mut digest = [0; DIGEST_SIZE_BYTE];
        for (block, integer) in digest.chunks_exact_mut(4).zip(self.state) {
            block.copy_from_slice(&integer.to_be_bytes());
        }

        digest
    }
}

/* -------------------------------------------------------------------------------- */

/// Secure Hash Algorithm 1 ([SHA-1](https://en.wikipedia.org/wiki/SHA-1))
///
/// # Example
///
/// ```
/// use cryptography::hash::{Sha1, Digest};
///
/// let message = b"The quick brown fox jumps over the lazy dog";
/// let hash = [ 0x2f, 0xd4, 0xe1, 0xc6, 0x7a, 0x2d, 0x28, 0xfc, 0xed, 0x84,
///              0x9e, 0xe1, 0xbb, 0x76, 0xe7, 0x39, 0x1b, 0x93, 0xeb, 0x12, ];
/// let mut hasher = Sha1::new();
/// hasher.update_in_place(message);
/// let result = hasher.digest();
/// assert_eq!(result, hash);
///
/// let message = b"The quick brown fox jumps over the lazy cog";
/// let hash = [ 0xde, 0x9f, 0x2c, 0x7f, 0xd2, 0x5e, 0x1b, 0x3a, 0xfa, 0xd3,
///              0xe8, 0x5a, 0x0b, 0xd1, 0x7d, 0x9b, 0x10, 0x0d, 0xb4, 0xb3, ];
/// let mut result = [0u8; 20];
/// Sha1::new().update(message).digest_into(&mut result);
/// assert_eq!(result, hash);
///
/// ```
pub type Sha1 = Hasher<Sha1Core, DIGEST_SIZE_BIT>;
