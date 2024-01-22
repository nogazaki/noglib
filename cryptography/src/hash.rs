//! Collection of [cryptographic hash functions]
//!
//! [cryptographic hash functions]: https://en.wikipedia.org/wiki/Cryptographic_hash_function

use crate::utils::error_types::InsufficientMemoryError;

/// Types that calculate fixed-sized digests
pub trait DigestUser {
    /// Digest size in bytes
    const DIGEST_SIZE: usize;
}

/// Functionalities of types that calculate fixed-sized digests
pub trait Digest: DigestUser {
    /// Create a new instance
    fn new() -> Self;
    /// Reset this instance
    fn reset(&mut self);

    /// Update this instance using `data`, chain-able
    #[must_use]
    fn update(self, data: &(impl AsRef<[u8]> + ?Sized)) -> Self;
    /// Update this instance in-place using `data`
    fn update_in_place(&mut self, data: &(impl AsRef<[u8]> + ?Sized));

    /// Finalize and return digest, consume this instance
    fn digest(self) -> [u8; Self::DIGEST_SIZE];
    /// Finalize digest into provided buffer, consume this instance
    ///
    /// # Errors
    /// - `InsufficientMemoryError` when `out` is not large enough to hold the digest
    fn digest_into(self, out: &mut impl AsMut<[u8]>) -> Result<(), InsufficientMemoryError>;
    /// Finalize and return digest, reset this instance
    fn digest_reset(&mut self) -> [u8; Self::DIGEST_SIZE];
    /// Finalize digest into provided buffer, reset this instance
    ///
    /// # Errors
    /// - `InsufficientMemoryError` when `out` is not large enough to hold the digest
    fn digest_into_reset(&mut self, out: &mut impl AsMut<[u8]>) -> Result<(), InsufficientMemoryError>;
}

/* -------------------------------------------------------------------------------- */

mod hasher;

mod sha1;
pub use sha1::Sha1;

mod sha2;
pub use sha2::{Sha224, Sha256};
