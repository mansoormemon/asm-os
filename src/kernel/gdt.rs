use lazy_static::lazy_static;
use x86_64::addr::VirtAddr;
use x86_64::instructions::segmentation::{CS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

use crate::kernel::memory;

/// Index of stack for double fault exceptions in the IST.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// Task State Segment (TSS)
//
// A Task State Segment is a binary data structure specific to the IA-32 and x86-64 architectures.
// The TSS was used to hold various pieces of information (e.g., processor register state) about a
// task in 32-bit mode and was, for example, used for hardware context switching. However, hardware
// context switching is no longer supported in 64-bit mode and the format of the TSS has changed
// completely.
//
// On x86_64, the TSS no longer holds any task-specific information at all. Instead, it holds two
// stack tables and an I/O port permissions bitmap:
//
// 1. Privilege Stack Table (PST)
// 2. Interrupt Stack Table (IST).
//
// The TSS is used to change the stack pointer after an interrupt or permission level is changed.
//
// OS Dev Wiki: https://wiki.osdev.org/Global_Descriptor_Table

lazy_static! {
    /// A global interface for Task State Segment (TSS).
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Set up a separate stack for double fault exceptions to avoid a triple fault exception,
        // which will reboot the machine. A triple fault exception is triggered if the stack is full
        // and the guard page is hit.
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // Create a stack of 8 pages (32 KiB).
            const STACK_SIZE: usize = 8 * memory::PAGE_SIZE;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_begin = VirtAddr::from_ptr(unsafe {&STACK});
            let stack_end = stack_begin + STACK_SIZE;
            stack_end
        };

        tss
    };
}

/// Selectors.
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

// Global Descriptor Table (GDT)
//
// The Global Descriptor Table (GDT) is a relic that was used for memory segmentation before paging
// became the de facto standard. However, it is still needed in 64-bit mode for various things,
// such as kernel/user mode configuration or TSS loading.
//
// The GDT is a structure that contains the segments of the program. It was used on older
// architectures to isolate programs from each other before paging became the standard.
//
// OS Dev Wiki: https://wiki.osdev.org/Global_Descriptor_Table

lazy_static! {
    /// A global interface for Global Descriptor Table (GDT).
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors{code_selector, tss_selector})
    };
}

/// Initializes the GDT.
pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
