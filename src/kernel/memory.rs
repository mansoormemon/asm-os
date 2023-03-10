use core::sync::atomic::{AtomicU64, Ordering};

use bootloader::BootInfo;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB, Translate};

// PAGING
//
// Paging is a system which allows each process to see a full virtual address space, without
// actually requiring the full amount of physical memory to be available or present. 32-bit x86
// processors support 32-bit virtual addresses and 4-GiB virtual address spaces, and current 64-bit
// processors support 48-bit virtual addressing and 256-TiB virtual address spaces.
//
// Paging in long mode is similar to that of 32-bit paging, except Physical Address Extension (PAE)
// is required. Registers CR2 and CR3 are extended to 64 bits. Instead of just having to utilize 3
// levels of page maps: page directory pointer table, page directory, and page table, a
// fourth page-map table is used: the level-4 page map table (PML4). This allows a processor to map
// 48-bit virtual addresses to 52-bit physical addresses. The PML4 contains 512 64-bit entries of
// which each may point to a lower-level page map table.
//
// OS Dev Wiki: https://wiki.osdev.org/Paging

/// Size of page.
pub const PAGE_SIZE: usize = 4096;

/// Physical memory offset in the virtual space.
static PHYS_MEM_OFFSET: AtomicU64 = AtomicU64::new(u64::MAX);

/// Initializes and returns the L4 page table.
pub fn init(boot_info: &'static BootInfo) {
    PHYS_MEM_OFFSET.store(boot_info.physical_memory_offset, Ordering::Relaxed);
}

/// Returns physical memory offset.
pub fn physical_memory_offset() -> u64 {
    PHYS_MEM_OFFSET.load(Ordering::Relaxed)
}

/// Returns active L4 page table set up by the bootloader to enable paging.
unsafe fn get_active_l4_table() -> &'static mut PageTable {
    let (l4_page_table, _) = Cr3::read();
    let phys_mem_offset = VirtAddr::new(physical_memory_offset());

    let phys_addr = l4_page_table.start_address();
    let virt_addr = phys_mem_offset + (phys_addr.as_u64());
    let page_table: *mut PageTable = virt_addr.as_mut_ptr();

    &mut *page_table
}

/// Returns the Offset Page Table.
pub unsafe fn mapper() -> OffsetPageTable<'static> {
    let l4_table = get_active_l4_table();
    let phys_mem_offset = VirtAddr::new(physical_memory_offset());

    OffsetPageTable::new(l4_table, phys_mem_offset)
}

/// Boot Info Frame Allocator.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Initializes the boot info frame allocator.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns the physical memory's usable frames.
    fn usable_frames(&self) -> impl Iterator<Item=PhysFrame> {
        let regions = self.memory_map.iter();
        // Filter usable regions.
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(PAGE_SIZE));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Translates physical address into virtual address.
pub fn phys_to_virt_addr(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_u64()) + PHYS_MEM_OFFSET.load(Ordering::Relaxed)
}

/// Translates virtual address into physical address.
pub fn virt_to_phys_addr(addr: VirtAddr) -> Option<PhysAddr> {
    let mapper = unsafe { mapper() };
    mapper.translate_addr(addr)
}
