// MIT License
//
// Copyright (c) 2023 Mansoor Ahmed Memon
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use alloc::alloc::Layout;

use bootloader::BootInfo;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::{FrameAllocator, Mapper};
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;

pub use bump::BumpAllocator;
pub use linked_list::LinkedListAllocator;
pub use pool::PoolAllocator;

use crate::krnl::memory;
use crate::success;

mod bump;
mod linked_list;
mod pool;

////////////////
// Attributes
////////////////

/// Start address of the the heap in the virtual space.
pub const HEAP_START: usize = 0x4444_4444_0000;
/// Size of heap.
pub const HEAP_SIZE: usize = 0x100000;
/// End address of heap in the virtual space.
pub const HEAP_END: usize = HEAP_START + HEAP_SIZE;

///////////////////////
// Global Interfaces
///////////////////////

/// A global interface for memory allocator.
#[global_allocator]
static ALLOCATOR: Locked<PoolAllocator> = Locked::new(PoolAllocator::new());

//////////////
/// Locked
//////////////
pub(crate) struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    /// Creates a new object.
    pub(crate) const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    /// Locks the object.
    pub(crate) fn lock(&self) -> MutexGuard<A> { self.inner.lock() }
}

/// A handler for allocation errors.
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! { panic!("allocation failure: {:?}", layout) }

///////////////
// Utilities
///////////////

/// Initializes the heap using a memory mapper and frame allocator.
pub(crate) fn init(boot_info: &'static BootInfo) {
    let mut mapper = unsafe { memory::mapper() };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::new(&boot_info.memory_map) };

    init_heap(&mut mapper, &mut frame_allocator).expect("failed to initialize heap");
}

/// Initializes the heap.
fn init_heap(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = VirtAddr::new(HEAP_END as u64);
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Map each page to a physical frame.
    for page in page_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe { ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE) };
    success!("Allocator initialized");

    Ok(())
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Note: Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize { (addr + align - 1) & !(align - 1) }
