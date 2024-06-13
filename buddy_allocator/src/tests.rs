use super::*;
use core::mem::{align_of, size_of_val};

// Ensure that a byte array is align to this size, which enables it to be added to the heap as a full block
#[repr(align(256))]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Aligned(u8);

const ORDERS: usize = align_of::<Aligned>().trailing_zeros() as usize - BASE_ORDER + 1;

#[test]
#[allow(clippy::shadow_unrelated)]
fn test_add_memory() {
    let aligned_pool = [Aligned(0); 2];
    let pool_addr = aligned_pool.as_ptr() as *mut u8;
    let pool_size = size_of_val(&aligned_pool);

    let allocator = BuddyAllocator::<ORDERS>::new();
    // Smaller than smallest pool that can be added
    let added = unsafe { allocator.add_memory(pool_addr, MIN_BLOCK_SIZE - 1) };
    assert_eq!(added, 0);
    // Smallest pool that can be added
    let added = unsafe { allocator.add_memory(pool_addr, MIN_BLOCK_SIZE) };
    assert_eq!(added, MIN_BLOCK_SIZE);

    let allocator = BuddyAllocator::<ORDERS>::new();
    // Add everything
    let added = unsafe { allocator.add_memory(pool_addr, pool_size) };
    assert_eq!(added, pool_size);

    const ALL_BLOCKS_POOL_SIZE: usize = MIN_BLOCK_SIZE * ((2 << (ORDERS - 1)) - 1);
    let allocator = BuddyAllocator::<ORDERS>::new();
    // Add all block sizes
    let added = unsafe { allocator.add_memory(pool_addr, ALL_BLOCKS_POOL_SIZE) };
    assert_eq!(added, ALL_BLOCKS_POOL_SIZE);
}

#[test]
#[allow(clippy::shadow_unrelated)]
fn test_memory_allocation() {
    let aligned_pool = [Aligned(0)];
    let pool_addr = aligned_pool.as_ptr() as *mut u8;
    let pool_size = size_of_val(&aligned_pool);

    let allocator = BuddyAllocator::<ORDERS>::new();
    // Add one biggest block
    let added = unsafe { allocator.add_memory(pool_addr, pool_size) };
    assert_eq!(added, allocator.get_max_block_size());

    // Request 1 bytes, which will allocate `MIN_LEAF_SIZE` bytes
    let layout = Layout::array::<u8>(1).unwrap();

    for offset in (0..pool_size).step_by(MIN_BLOCK_SIZE) {
        let result = unsafe { allocator.get_memory(layout) };
        assert!(result.is_some_and(
            |ptr| ptr.as_ptr() as *mut u8 == unsafe { pool_addr.add(offset) } && ptr.len() == MIN_BLOCK_SIZE
        ));
    }
    // No more memory to allocate
    unsafe { assert!(allocator.get_memory(layout).is_none()) };

    // Deallocate every requested block
    for offset in (0..pool_size).step_by(MIN_BLOCK_SIZE) {
        let ptr = NonNull::new(unsafe { pool_addr.add(offset) }).expect("No null pointer are created");
        unsafe { allocator.return_memory(ptr, layout) };
    }

    // Allocate maximum block size
    let layout = Layout::array::<u8>(BuddyAllocator::<ORDERS>::MAX_BLOCK_SIZE).unwrap();
    let result = unsafe { allocator.get_memory(layout) };
    assert!(result.is_some_and(|ptr| (ptr.as_ptr() as *mut u8 == pool_addr) && (ptr.len() == layout.size())));
    // No more memory to allocate
    unsafe { assert!(allocator.get_memory(layout).is_none()) };
}
