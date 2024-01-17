//! A naive implementation of the buddy memory allocator

#![no_std]

use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ptr::NonNull;

mod header;
use header::BlockHeader;

/// Minimal block size allocatable
pub const MIN_BLOCK_SIZE: usize = size_of::<BlockHeader>();
/// Order of the minimal block size allocatable
const BASE_ORDER: usize = MIN_BLOCK_SIZE.trailing_zeros() as usize;

/// The buddy allocator
///
/// # Usage
///
/// Create a heap and add a memory region to it:
/// ```
/// use buddy_allocator::*;
///
/// let mut allocator = Allocator::<5>::new();
/// let pool = [0u8; 256];
/// let added_memory_size = unsafe { allocator.add_memory(&pool) };
/// ```
#[derive(Debug)]
pub struct Allocator<'a, const ORDERS: usize> {
    /// List of pointers to the first free block at each level
    free_list: [BlockHeader; ORDERS],
    /// Total memory size allocatable
    total_size: usize,
    /// Phantom data, keeping memory pools added to this allocator valid
    _pd: PhantomData<&'a [u8]>,
}
impl<const ORDERS: usize> Allocator<'_, ORDERS> {
    /// Maximum block size allocatable, accessible with type
    pub const MAX_BLOCK_SIZE: usize = 1 << (ORDERS + BASE_ORDER - 1);

    /// Maximum block size allocatable, accessible with instance
    pub const fn get_max_block_size(&self) -> usize {
        Self::MAX_BLOCK_SIZE
    }

    /// Create an allocator with no memory yet
    pub const fn new() -> Self {
        Allocator {
            free_list: [BlockHeader::new(); ORDERS],
            total_size: 0,
            _pd: PhantomData,
        }
    }

    /// Add a memory pool to the heap of this allocator
    ///
    /// # Safety
    /// This method takes ownership of `pool`, so it should not be used again, at least until the allocator was dropped.
    ///
    /// # Note
    /// `pool` is not required to be mutable to help prevent write access in its scope,
    /// but would definitely be modified by the allocator.
    pub unsafe fn add_memory(&mut self, pool: &'_ [u8]) -> usize {
        let mut start = pool.as_ptr() as usize;
        let mut end = start + pool.len();

        // Ensure alignment
        start = (start + MIN_BLOCK_SIZE - 1) & (!MIN_BLOCK_SIZE + 1);
        end &= !MIN_BLOCK_SIZE + 1;

        let mut added = 0;
        while start + MIN_BLOCK_SIZE <= end {
            // Block must be properly align before accommodating largest possible block that the allocator support
            let size = Self::MAX_BLOCK_SIZE
                .min(start & (!start + 1)) // Maximum alignment of current address
                .min((end - start + 1).next_power_of_two() >> 1); // Maximum block size fits in remaining memory
            let order = size.trailing_zeros() as usize - BASE_ORDER;

            self.free_list[order].push(start as *mut _);
            added += size;
            start += size;
        }

        self.total_size += added;
        added
    }

    /// Split a memory block with order of (`index` + `BASE_ORDER`) into 2 smaller block with order of (`index` + `BASE_ORDER` - 1)
    #[inline]
    fn split_block(&mut self, index: usize) {
        if let Some(block) = self.free_list[index].pop_next() {
            let block_size = 1 << (index + BASE_ORDER - 1);
            // SAFETY: pointer is within the larger block
            let buddy = unsafe { (block as *mut u8).add(block_size) } as *mut BlockHeader;
            // SAFETY: pointer is within the larger block, its size does not overflow
            unsafe { self.free_list[index - 1].push(buddy) };
            unsafe { self.free_list[index - 1].push(block) };
        }
    }

    /// Allocate a piece of memory from the pool, satisfying `layout` requirements
    pub fn allocate(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let size = MIN_BLOCK_SIZE
            .max(layout.size().next_power_of_two())
            .max(layout.align());
        let index = size.trailing_zeros() as usize - BASE_ORDER;

        for i in index..ORDERS {
            // Find smallest order that is available for allocation
            if self.free_list[i].is_tail() {
                continue;
            }

            // Split the block if it is larger than requested, until a block of requested size is available
            for j in (index + 1..i + 1).rev() {
                self.split_block(j);
            }

            break;
        }

        self.free_list[index]
            .pop_next()
            .and_then(|ptr| NonNull::new(ptr as *mut u8))
    }

    /// Deallocate a piece of memory
    pub fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = MIN_BLOCK_SIZE
            .max(layout.size().next_power_of_two())
            .max(layout.align());
        let mut index = size.trailing_zeros() as usize - BASE_ORDER;

        let mut block = ptr.as_ptr() as usize;
        for list in self.free_list.iter_mut().skip(index) {
            let buddy = block ^ (1 << (index + BASE_ORDER));
            let mut has_buddy = false;

            for node in list.iter_mut().skip(1) {
                if node as usize != buddy {
                    continue;
                }

                unsafe { (*node).pop() };
                has_buddy = true;
                break;
            }

            if has_buddy {
                block = block.min(buddy);
                index += 1;
            } else {
                break;
            }
        }

        unsafe { self.free_list[index].push(block as *mut _) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of_val};
    use core::slice::from_raw_parts;

    // Ensure that a byte array is align to this size, which enables it to be added to the heap as a full block
    #[repr(align(256))]
    #[derive(Clone, Copy)]
    struct Aligned(u8);

    const ORDERS: usize = align_of::<Aligned>().trailing_zeros() as usize - BASE_ORDER + 1;

    #[test]
    #[allow(clippy::shadow_unrelated)]
    fn test_add_memory() {
        let aligned_pool = [Aligned(0); 2];
        let pool = unsafe {
            from_raw_parts(
                &aligned_pool as *const _ as *const u8,
                size_of_val(&aligned_pool),
            )
        };

        let mut allocator = Allocator::<ORDERS>::new();
        // Smaller than smallest pool that can be added
        let added = unsafe { allocator.add_memory(&pool[..MIN_BLOCK_SIZE - 1]) };
        assert_eq!(added, 0);
        // Smallest pool that can be added
        let added = unsafe { allocator.add_memory(&pool[..MIN_BLOCK_SIZE]) };
        assert_eq!(added, MIN_BLOCK_SIZE);

        let mut allocator = Allocator::<ORDERS>::new();
        // Add biggest blocks
        let added = unsafe { allocator.add_memory(pool) };
        assert_eq!(added, pool.len());

        const ALL_BLOCKS_POOL_SIZE: usize = MIN_BLOCK_SIZE * ((2 << (ORDERS - 1)) - 1);
        let mut allocator = Allocator::<ORDERS>::new();
        // Add all block sizes
        let added = unsafe { allocator.add_memory(&pool[..ALL_BLOCKS_POOL_SIZE]) };
        assert_eq!(added, ALL_BLOCKS_POOL_SIZE);
    }

    #[test]
    #[allow(clippy::shadow_unrelated)]
    fn test_memory_allocation() {
        let aligned_pool = Aligned(0);
        let pool = unsafe {
            from_raw_parts(
                &aligned_pool as *const _ as *const u8,
                size_of_val(&aligned_pool),
            )
        };

        let mut allocator = Allocator::<ORDERS>::new();
        // Add one biggest block
        let added = unsafe { allocator.add_memory(pool) };
        assert_eq!(added, allocator.get_max_block_size());

        // Request 1 bytes, which will allocate `MIN_LEAF_SIZE` bytes
        let layout = Layout::array::<u8>(1).unwrap();
        for slice in pool.chunks_exact(MIN_BLOCK_SIZE) {
            let result = allocator.allocate(layout);
            assert!(result.is_some_and(|ptr| ptr.as_ptr() as *const _ == slice.as_ptr()));
        }

        // No more memory to allocate
        assert!(allocator.allocate(layout).is_none());

        // Deallocate every requested block
        for slice in pool.chunks_exact(MIN_BLOCK_SIZE) {
            let ptr = NonNull::new(slice.as_ptr() as *mut _).unwrap();
            allocator.deallocate(ptr, layout);
        }

        // Allocate maximum block size
        let layout = Layout::array::<u8>(Allocator::<ORDERS>::MAX_BLOCK_SIZE).unwrap();
        let result = allocator.allocate(layout);
        assert!(result.is_some_and(|ptr| ptr.as_ptr() as *const _ == &pool[0]));
    }
}
