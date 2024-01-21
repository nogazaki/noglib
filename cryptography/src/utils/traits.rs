//! Commonly used traits

/// Types that use key
pub trait KeyUser {
    /// Key size in bytes
    const KEY_SIZE: usize;

    /// Initialize an instance with a key
    fn init(key: &[u8; Self::KEY_SIZE]) -> Self;
}

/// Types that operate on blocks
pub trait BlockUser {
    /// Block size in bytes
    const BLOCK_SIZE: usize;

    /// Split a slice in to blocks and leftover
    fn split_blocks(data: &[u8]) -> (&[[u8; Self::BLOCK_SIZE]], &[u8]) {
        let num_of_blocks = data.len() / Self::BLOCK_SIZE;
        let tail_len = data.len() % Self::BLOCK_SIZE;

        use core::slice::from_raw_parts;
        unsafe {
            let blocks_ptr = data.as_ptr() as *const [u8; Self::BLOCK_SIZE];
            let tail_ptr = blocks_ptr.add(num_of_blocks) as *const u8;
            (
                from_raw_parts(blocks_ptr, num_of_blocks),
                from_raw_parts(tail_ptr, tail_len),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct Block;
    impl BlockUser for Block {
        const BLOCK_SIZE: usize = 16;
    }

    #[test]
    #[allow(clippy::shadow_unrelated)]
    fn test_split_blocks() {
        const NUM_OF_BLOCKS: usize = 2;
        const TAIL_LEN_BYTES: usize = 2;

        const _CONST_CHECK: () = assert!(TAIL_LEN_BYTES < Block::BLOCK_SIZE);

        let data = [0_u8; Block::BLOCK_SIZE * NUM_OF_BLOCKS + TAIL_LEN_BYTES];

        let (blocks, tail) = Block::split_blocks(&data);
        assert_eq!(blocks.len(), NUM_OF_BLOCKS);
        assert_eq!(tail.len(), TAIL_LEN_BYTES);

        let (blocks, tail) = Block::split_blocks(&data[..Block::BLOCK_SIZE * NUM_OF_BLOCKS]);
        assert_eq!(blocks.len(), NUM_OF_BLOCKS);
        assert_eq!(tail.len(), 0);

        let (blocks, tail) = Block::split_blocks(&data[..Block::BLOCK_SIZE * NUM_OF_BLOCKS - 1]);
        assert_eq!(blocks.len(), NUM_OF_BLOCKS - 1);
        assert_eq!(tail.len(), Block::BLOCK_SIZE - 1);
    }
}
