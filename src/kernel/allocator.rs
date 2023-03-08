use alloc::alloc::Layout;

use bootloader::BootInfo;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;

pub use bump::BumpAllocator;
pub use linked_list::LinkedListAllocator;
pub use pool::PoolAllocator;

use crate::aux::units::Unit;
use crate::kernel::memory;

mod bump;
mod linked_list;
mod pool;

/// Locked
pub struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    /// Creates a new Locked object.
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    /// Locks the object.
    pub fn lock(&self) -> MutexGuard<A> {
        self.inner.lock()
    }
}

/// Start address of the the heap in the virtual space.
pub const HEAP_START: usize = 0x4444_4444_0000;
/// Size of heap.
pub const HEAP_SIZE: usize = Unit::MiB as usize;
/// End address of heap in the virtual space.
pub const HEAP_END: usize = HEAP_START + HEAP_SIZE;

/// A global interface for memory allocator.
#[global_allocator]
static ALLOCATOR: Locked<PoolAllocator> = Locked::new(PoolAllocator::new());

/// A handler for allocation errors.
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

/// Intializes memory heap using mapper and frame allocator.
pub fn init(boot_info: &'static BootInfo) {
    let mut mapper = unsafe { memory::mapper() };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

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

    Ok(())
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
