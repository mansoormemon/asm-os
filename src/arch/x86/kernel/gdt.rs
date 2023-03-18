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

use lazy_static::lazy_static;
use x86_64::addr::VirtAddr;
use x86_64::instructions;
use x86_64::instructions::segmentation::CS;
use x86_64::instructions::segmentation::Segment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

use crate::arch::x86::kernel::memory;
use crate::success;

////////////////
// Attributes
////////////////

// A stack size of 8 pages (32 KiB).
pub const STACK_SIZE: usize = 8 * memory::PAGE_SIZE;

/////////////
/// Stack
/////////////
#[repr(u16)]
pub enum Stack {
    DoubleFault = 0x0,
}

////////////////
// Interfaces
////////////////

lazy_static! {
    /// Task State Segment (TSS)
    ///
    /// A Task State Segment is a binary data structure specific to the IA-32 and x86-64 architectures.
    /// The TSS was used to hold various pieces of information (e.g., processor register state) about a
    /// task in 32-bit mode and was, for example, used for hardware context switching. However, hardware
    /// context switching is no longer supported in 64-bit mode and the format of the TSS has changed
    /// completely.
    ///
    /// On x86, the TSS no longer holds any task-specific information at all. Instead, it holds two
    /// stack tables and an I/O port permissions bitmap:
    ///
    /// 1. Privilege Stack Table (PST)
    /// 2. Interrupt Stack Table (IST).
    ///
    /// The TSS is used to change the stack pointer after an interrupt or permission level is changed.
    ///
    /// OS Dev Wiki: https://wiki.osdev.org/Global_Descriptor_Table
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Set up a separate stack for double fault exceptions to avoid a triple fault exception,
        // which will reboot the machine. A triple fault exception is triggered if the stack is full
        // and the guard page is hit.
        tss.interrupt_stack_table[Stack::DoubleFault as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_begin = VirtAddr::from_ptr(unsafe {&STACK});
            let stack_end = stack_begin + STACK_SIZE;
            stack_end
        };

        tss
    };
}

lazy_static! {
    /// Global Descriptor Table (GDT)
    ///
    /// The Global Descriptor Table (GDT) is a relic that was used for memory segmentation before paging
    /// became the de facto standard. However, it is still needed in 64-bit mode for various things,
    /// such as kernel/user mode configuration or TSS loading.
    ///
    /// The GDT is a structure that contains the segments of the program. It was used on older
    /// architectures to isolate programs from each other before paging became the standard.
    ///
    /// OS Dev Wiki: https://wiki.osdev.org/Global_Descriptor_Table
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors{code_selector, tss_selector})
    };
}

/////////////////
/// Selectors
/////////////////
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

///////////////
// Utilities
///////////////

/// Initializes the GDT.
pub(crate) fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        instructions::tables::load_tss(GDT.1.tss_selector);
    }
    success!("GDT initialized");
}
