#![feature(allocator_api)]

use buddy_allocator::BuddyAllocator;
use spin as _;

#[test]
fn main() {
    let allocator = BuddyAllocator::<5>::new();
    let memory_pool = [0_u8; 512];

    unsafe { allocator.add_memory(memory_pool.as_ptr() as *mut u8, memory_pool.len()) };

    let data = [1; 100];
    let mut a: Vec<u8, _> = Vec::with_capacity_in(100, allocator);
    assert_eq!(a.len(), 0);

    a.extend_from_slice(&data);
    assert_eq!(a.len(), 100);

    assert_eq!(a.iter().sum::<u8>(), 100);
}
