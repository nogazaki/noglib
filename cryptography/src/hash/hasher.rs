//! A generic wrapper around hashing types to truncate its output

use crate::hash::{Digest, DigestUser};
use crate::utils::{error_types::InsufficientMemoryError, traits::BlockUser};

/// Functionalities of a hasher core
pub trait HasherCore: DigestUser {
    /// Create a new instance, based on the truncated digest size
    fn new(truncated_digest_len_bit: usize) -> Self;
    /// Compress the data
    fn compress(&mut self, data: &[u8]);
    /// Finalize and return the full, un-truncated digest
    fn finalize(&mut self) -> [u8; Self::DIGEST_SIZE];
}

/// Wrapper around hashing types to truncate its output
#[derive(Debug)]
pub struct Hasher<Core: HasherCore, const DIGEST_SIZE_BIT: usize> {
    /// The hashing engine
    core: Core,
}

impl<Core: HasherCore + BlockUser, const DIGEST_SIZE_BIT: usize> BlockUser for Hasher<Core, DIGEST_SIZE_BIT> {
    const BLOCK_SIZE: usize = Core::BLOCK_SIZE;
}

impl<Core: HasherCore, const DIGEST_SIZE_BIT: usize> DigestUser for Hasher<Core, DIGEST_SIZE_BIT> {
    const DIGEST_SIZE: usize = DIGEST_SIZE_BIT >> 3;
}

impl<Core: HasherCore, const DIGEST_SIZE_BIT: usize> Digest for Hasher<Core, DIGEST_SIZE_BIT>
where
    [(); Core::DIGEST_SIZE]:,
{
    fn new() -> Self {
        Self {
            core: Core::new(DIGEST_SIZE_BIT),
        }
    }
    fn reset(&mut self) {
        *self = Self::new();
    }

    fn update(mut self, data: &(impl AsRef<[u8]> + ?Sized)) -> Self {
        self.core.compress(data.as_ref());
        self
    }
    fn update_in_place(&mut self, data: &(impl AsRef<[u8]> + ?Sized)) {
        self.core.compress(data.as_ref());
    }

    fn digest(mut self) -> [u8; Self::DIGEST_SIZE] {
        let full_digest = self.core.finalize();
        full_digest[..Self::DIGEST_SIZE].try_into().unwrap()
    }
    fn digest_into(mut self, out: &mut impl AsMut<[u8]>) -> Result<(), InsufficientMemoryError> {
        if out.as_mut().len() < Self::DIGEST_SIZE {
            return Err(InsufficientMemoryError {});
        }

        let full_digest = self.core.finalize();
        out.as_mut()[..Self::DIGEST_SIZE].copy_from_slice(&full_digest[..Self::DIGEST_SIZE]);

        Ok(())
    }
    fn digest_reset(&mut self) -> [u8; Self::DIGEST_SIZE] {
        let full_digest = self.core.finalize();
        self.reset();

        full_digest[..Self::DIGEST_SIZE].try_into().unwrap()
    }
    fn digest_into_reset(&mut self, out: &mut impl AsMut<[u8]>) -> Result<(), InsufficientMemoryError> {
        if out.as_mut().len() < Self::DIGEST_SIZE {
            return Err(InsufficientMemoryError {});
        }

        let full_digest = self.core.finalize();
        self.reset();
        out.as_mut()[..Self::DIGEST_SIZE].copy_from_slice(&full_digest[..Self::DIGEST_SIZE]);

        Ok(())
    }
}
