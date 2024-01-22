//! Secure Hash Algorithm 2 ([SHA-2](https://en.wikipedia.org/wiki/SHA-2))

use super::hasher::{Hasher, HasherCore};

/// SHA-224 digest size in bits
const SHA224_DIGEST_SIZE_BIT: usize = 224;
/// SHA-256 digest size in bits
const SHA256_DIGEST_SIZE_BIT: usize = 256;
// /// SHA-384 digest size in bits
// const SHA384_DIGEST_SIZE_BIT: usize = 384;
// /// SHA-512 digest size in bits
// const SHA512_DIGEST_SIZE_BIT: usize = 512;

mod core256;

/* -------------------------------------------------------------------------------- */

/// Secure Hash Algorithm 2 ([SHA-2](https://en.wikipedia.org/wiki/SHA-2)), SHA-224 variant
///
/// # Example
///
/// ```
/// use cryptography::hash::{Sha224, Digest};
///
/// let hash = [ 0xd1, 0x4a, 0x02, 0x8c, 0x2a, 0x3a, 0x2b, 0xc9, 0x47, 0x61, 0x02, 0xbb, 0x28, 0x82,
///              0x34, 0xc4, 0x15, 0xa2, 0xb0, 0x1f, 0x82, 0x8e, 0xa6, 0x2a, 0xc5, 0xb3, 0xe4, 0x2f, ];
/// let result = Sha224::new().update("").digest();
/// assert_eq!(result, hash);
///
/// ```
pub type Sha224 = Hasher<core256::Sha256Core, SHA224_DIGEST_SIZE_BIT>;

/// Secure Hash Algorithm 2 ([SHA-2](https://en.wikipedia.org/wiki/SHA-2)), SHA-256 variant
///
/// # Example
///
/// ```
/// use cryptography::hash::{Sha256, Digest};
///
/// let hash = [ 0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
///              0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55, ];
/// let result = Sha256::new().update("").digest();
/// assert_eq!(result, hash);
///
/// ```
pub type Sha256 = Hasher<core256::Sha256Core, SHA256_DIGEST_SIZE_BIT>;
