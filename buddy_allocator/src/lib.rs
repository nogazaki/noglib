//! A naive implementation of the buddy memory allocator

#![no_std]

use core::alloc::Layout;
use core::{
    marker::PhantomData,
    mem::size_of,
    ptr::{null_mut, NonNull},
    slice::from_raw_parts_mut,
};
use spin::Mutex;

mod header;
use header::BlockHeader;

#[cfg(test)]
mod tests;

/* -------------------------------------------------------------------------------- */

/// Minimal block size allocatable
const MIN_BLOCK_SIZE: usize = size_of::<BlockHeader>();
/// Order of the minimal block size allocatable
const BASE_ORDER: usize = MIN_BLOCK_SIZE.trailing_zeros() as usize;

/// Get order of an allocator for a max block size
#[inline(always)]
pub const fn order_from_max_block_size(max_block_size: usize) -> usize {
    max_block_size.trailing_zeros() as usize - BASE_ORDER + 1
}

/* -------------------------------------------------------------------------------- */

/// The buddy allocator
///
/// # Usage
///
/// Create a heap and add a memory region to it:
/// ```
/// use buddy_allocator::*;
///
/// let mut allocator = BuddyAllocator::<5>::new();
/// let pool = [0u8; 256];
/// let added_memory_size = unsafe { allocator.add_memory(&pool as *const _ as *mut u8, pool.len()) };
///
/// let layout = core::alloc::Layout::array::<u8>(1).unwrap();
/// let result = unsafe { allocator.get_memory(layout) };
/// assert!(result.is_some());
/// ```
#[derive(Debug)]
pub struct BuddyAllocator<'a, const ORDERS: usize> {
    /// List of pointers to the first free block at each level
    free_list: Mutex<[BlockHeader; ORDERS]>,
    /// Phantom data, keeping memory pools added to this allocator valid
    _pd: PhantomData<&'a [u8]>,
}

impl<'a, const ORDERS: usize> BuddyAllocator<'a, ORDERS> {
    /// Maximum block size allocatable, accessible with type
    pub const MAX_BLOCK_SIZE: usize = 1 << (ORDERS + BASE_ORDER - 1);

    /// Maximum block size allocatable, accessible with instance
    #[inline(always)]
    pub const fn get_max_block_size(&self) -> usize {
        Self::MAX_BLOCK_SIZE
    }

    /// Create an allocator with no memory yet
    pub const fn new() -> Self {
        BuddyAllocator {
            free_list: Mutex::new([BlockHeader::new(); ORDERS]),
            _pd: PhantomData,
        }
    }

    /// Add a memory pool to the heap of this allocator
    ///
    /// # Safety
    /// The caller must ensure that there is no reference that
    /// point to the contents of the `UnsafeCell`.
    pub unsafe fn add_memory(&self, pool_addr: *mut u8, pool_size: usize) -> usize {
        let mut start = pool_addr as usize;
        let mut end = start + pool_size;

        // Ensure alignment
        start = (start + MIN_BLOCK_SIZE - 1) & (!MIN_BLOCK_SIZE + 1);
        end &= !MIN_BLOCK_SIZE + 1;

        let mut free_list = self.free_list.lock();
        let mut added = 0;
        while start + MIN_BLOCK_SIZE <= end {
            // Block must be properly align before accommodating largest possible block that the allocator support
            let size = Self::MAX_BLOCK_SIZE
                .min(start & (!start + 1)) // Maximum alignment of current address
                .min((end - start + 1).next_power_of_two() >> 1); // Maximum block size fits in remaining memory
            let order = size.trailing_zeros() as usize - BASE_ORDER;

            free_list[order].push(start as *mut _);
            added += size;
            start += size;
        }

        added
    }

    /// Allocate a piece of memory from the pool, satisfying `layout` requirements
    /// # Safety
    pub unsafe fn get_memory(&self, layout: Layout) -> Option<NonNull<[u8]>> {
        let size = MIN_BLOCK_SIZE
            .max(layout.size().next_power_of_two())
            .max(layout.align());
        let index = size.trailing_zeros() as usize - BASE_ORDER;

        let mut free_list = self.free_list.lock();
        for i in index..ORDERS {
            // Find smallest order that is available for allocation
            if free_list[i].is_tail() {
                continue;
            }

            // Split the block if it is larger than requested, until a block of requested size is available
            for j in (index + 1..i + 1).rev() {
                if let Some(block) = free_list[j].pop_next() {
                    let block_size = 1 << (j + BASE_ORDER - 1);
                    // SAFETY: pointer is within the larger block
                    let buddy = (block as *mut u8).add(block_size) as *mut BlockHeader;

                    // SAFETY: pointer is within the larger block, its size does not overflow
                    free_list[j - 1].push(buddy);
                    free_list[j - 1].push(block);
                }
            }

            break;
        }

        free_list[index]
            .pop_next()
            .and_then(|ptr| NonNull::new(from_raw_parts_mut(ptr as *mut _, size)))
    }

    /// Deallocate a piece of memory
    /// # Safety
    pub unsafe fn return_memory(&self, ptr: NonNull<u8>, layout: Layout) {
        let size = MIN_BLOCK_SIZE
            .max(layout.size().next_power_of_two())
            .max(layout.align());
        let mut index = size.trailing_zeros() as usize - BASE_ORDER;

        let mut free_list = self.free_list.lock();
        let mut block = ptr.as_ptr() as usize;
        for list in free_list.iter_mut().rev().skip(1).rev().skip(index) {
            let buddy = block ^ (1 << (index + BASE_ORDER));
            let mut has_buddy = false;

            for node in list.iter_mut().skip(1) {
                if node as usize != buddy {
                    continue;
                }

                (*node).pop();
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

        free_list[index].push(block as *mut _);
    }
}

impl<const ORDERS: usize> Default for BuddyAllocator<'_, ORDERS> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<const ORDERS: usize> Sync for BuddyAllocator<'static, ORDERS> {}

/* -------------------------------------------------------------------------------- */

extern crate alloc;
use alloc::alloc::GlobalAlloc;

unsafe impl<const ORDERS: usize> GlobalAlloc for BuddyAllocator<'static, ORDERS> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.get_memory(layout).map_or(null_mut(), |ptr| ptr.as_ptr() as *mut _)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(ptr) = NonNull::new(ptr) {
            self.return_memory(ptr, layout);
        }
    }
}

// use alloc::alloc::Allocator;
// use core::alloc::AllocError;

// unsafe impl<const ORDERS: usize> Allocator for BuddyAllocator<'_, ORDERS> {
//     fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//         unsafe { self.get_memory(layout).ok_or(AllocError {}) }
//     }

//     unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
//         unsafe { self.return_memory(ptr, layout) }
//     }
// }
