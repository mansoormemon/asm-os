use core::{mem, ptr};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

use crate::nub::allocator::Locked;

/// List Node
struct ListNode {
    next: Option<&'static mut ListNode>,
}

/// Block size of available buckets.
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

/// Pool Allocator
pub struct PoolAllocator {
    buckets: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl PoolAllocator {
    /// Creates a new object.
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;

        PoolAllocator {
            buckets: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initializes the allocator.
    pub unsafe fn init(&mut self, heap_start: usize, heap_end: usize) {
        self.fallback_allocator.init(heap_start, heap_end);
    }

    /// Allocate memory using fallback allocator.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }

    /// Deallocate memory allocated by the fallback allocator.
    unsafe fn fallback_dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();
        self.fallback_allocator.deallocate(ptr, layout);
    }

    /// Returns the index of a suitable block size.
    fn list_index(layout: &Layout) -> Option<usize> {
        let required_block_size = layout.size().max(layout.align());
        BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
    }
}

unsafe impl GlobalAlloc for Locked<PoolAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        match PoolAllocator::list_index(&layout) {
            Some(index) => {
                match allocator.buckets[index].take() {
                    Some(node) => {
                        allocator.buckets[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => {
                allocator.fallback_alloc(layout)
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        match PoolAllocator::list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.buckets[index].take(),
                };

                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.buckets[index] = Some(&mut *new_node_ptr);
            }
            None => {
                allocator.fallback_dealloc(ptr, layout);
            }
        }
    }
}
