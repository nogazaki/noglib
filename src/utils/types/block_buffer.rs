//! A buffer that can be used by `crate::utils::traits::BlockUser` types to store and process arbitrarily sized data

use crate::utils::traits::BlockUser;

/// A buffer that can be used by `crate::utils::traits::BlockUser` types to store and process arbitrarily sized data
#[derive(Debug)]
pub struct BlockBuffer<const BLOCK_SIZE: usize> {
    /// Actual buffer storing the data
    buf: [u8; BLOCK_SIZE],
    /// Actual size of relevant data storing in `buf`
    pos: usize,
}

impl<const BLOCK_SIZE: usize> BlockUser for BlockBuffer<BLOCK_SIZE> {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl<const BLOCK_SIZE: usize> Default for BlockBuffer<BLOCK_SIZE> {
    fn default() -> Self {
        BlockBuffer {
            buf: [0_u8; BLOCK_SIZE],
            pos: 0,
        }
    }
}

impl<const BLOCK_SIZE: usize> BlockBuffer<BLOCK_SIZE>
where
    [(); Self::BLOCK_SIZE]:,
{
    /// Create a new block buffer
    pub fn new() -> Self {
        BlockBuffer::default()
    }

    /// Get remaining size in bytes of this buffer
    pub const fn get_remain(&self) -> usize {
        BLOCK_SIZE - self.get_pos()
    }

    /// Get pointer to the next unused byte of this buffer
    pub const fn get_pos(&self) -> usize {
        self.pos
    }

    /// Set the pointer to the next unused byte of this buffer
    fn set_pos_unchecked(&mut self, pos: usize) {
        self.pos = pos;
    }

    /// Parse a data slice, calling `processor` on the portion that fit into multiple blocks
    /// and store the remaining in this buffer.
    /// Any data that is currently in the buffer will be concatenate to the start of `data`
    pub fn process_data(&mut self, mut data: &[u8], mut processor: impl FnMut(&[[u8; BLOCK_SIZE]])) {
        let len = data.len();

        let pos = self.get_pos();
        let rem = self.get_remain();

        if len < rem {
            self.buf[pos..][..len].copy_from_slice(data);
            self.set_pos_unchecked(pos + len);

            return;
        }

        if pos != 0 {
            let (left, right) = data.split_at(rem);
            self.buf[pos..].copy_from_slice(left);
            processor(core::slice::from_ref(&self.buf));

            data = right;
        }

        let (blocks, tail) = Self::split_blocks(data);
        if !blocks.is_empty() {
            // SAFETY: `Self::BLOCK_SIZE` is `BLOCK_SIZE`
            processor(unsafe { core::mem::transmute(blocks) });
        }

        self.buf[..tail.len()].copy_from_slice(tail);
        self.buf[tail.len()..].fill(0);
        self.set_pos_unchecked(tail.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK_SIZE: usize = 5;
    const HALF_BLOCK_SIZE: usize = BLOCK_SIZE / 2;

    #[test]
    fn test_buffer_content() {
        let data = [255; BLOCK_SIZE * 2];
        let mut reference = [0; BLOCK_SIZE];

        let mut buffer = BlockBuffer::<BLOCK_SIZE>::new();
        buffer.process_data(&data[..HALF_BLOCK_SIZE], |_| {});
        reference[..HALF_BLOCK_SIZE].fill(255);
        assert_eq!(buffer.pos, HALF_BLOCK_SIZE);
        assert_eq!(buffer.buf, reference);

        buffer.process_data(&data[HALF_BLOCK_SIZE..][..BLOCK_SIZE - 1], |_| {});
        reference.fill(0);
        reference[..HALF_BLOCK_SIZE - 1].fill(255);
        assert_eq!(buffer.pos, HALF_BLOCK_SIZE - 1);
        assert_eq!(buffer.buf, reference);

        buffer.process_data(&data[HALF_BLOCK_SIZE + BLOCK_SIZE - 1..], |_| {});
        reference.fill(0);
        assert_eq!(buffer.pos, 0);
        assert_eq!(buffer.buf, reference);
    }

    #[test]
    fn test_processor() {
        let data = [1_u8; BLOCK_SIZE + HALF_BLOCK_SIZE];

        let mut buffer = BlockBuffer::<BLOCK_SIZE>::new();
        let mut sum = 0;

        buffer.process_data(&data[..HALF_BLOCK_SIZE], |blocks| {
            sum += blocks.iter().fold(0, |s, block| s + block.iter().sum::<u8>());
        });
        assert_eq!(sum, 0);

        buffer.process_data(&data[HALF_BLOCK_SIZE..], |blocks| {
            sum += blocks.iter().fold(0, |s, block| s + block.iter().sum::<u8>());
        });
        assert_eq!(sum as usize, BLOCK_SIZE);
    }
}
