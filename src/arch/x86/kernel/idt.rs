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

use instructions::port::Port;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{hlt_loop, println, success};
use crate::arch::x86::kernel::gdt;
use crate::arch::x86::kernel::pics;
use crate::arch::x86::kernel::pics::PICS_8259;

/// Maps the interrupt handler.
macro_rules! map_irq_handler {
    ($reference:ident, $handler:ident, $interrupt:expr) => {
        $reference[IRQ::index_to_pin($interrupt) as usize].set_handler_fn($handler);
    };
}

/// Generates the interrupt handler.
macro_rules! generate_irq_handler {
    ($handler:ident, $irq_idx:expr) => {
        extern "x86-interrupt" fn $handler(_stack_frame: InterruptStackFrame) {
            let irq_handlers = IRQ_HANDLERS.lock();
            irq_handlers[$irq_idx]();
            unsafe { PICS_8259.lock().notify_end_of_interrupt(IRQ::index_to_pin($irq_idx)); }
        }
    };
}

lazy_static! {
    /// List of all IRQ handlers.
    static ref IRQ_HANDLERS: Mutex<[fn(); pics::TOTAL_PIN_COUNT as usize]> = Mutex::new(
        [IRQ::default_handler; pics::TOTAL_PIN_COUNT as usize]
    );
}

lazy_static! {
    /// Interrupt Descriptor Table (IDT)
    ///
    /// The Interrupt Descriptor Table (IDT) is a data structure used by the x86 architecture to
    /// implement an interrupt vector table. The IDT is used by the processor to determine the correct
    /// response to interrupts and exceptions.
    ///
    /// Wikipedia: https://en.wikipedia.org/wiki/Interrupt_descriptor_table
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Set breakpoint handler.
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // Set double fault handler and stack index.
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::Stack::DoubleFault as u16);
        }

        // Set page fault handler.
        idt.page_fault.set_handler_fn(page_fault_handler);

        // Map interrupt handlers.
        map_irq_handler!(idt, irq_0x0_handler, 0x0);
        map_irq_handler!(idt, irq_0x1_handler, 0x1);
        map_irq_handler!(idt, irq_0x2_handler, 0x2);
        map_irq_handler!(idt, irq_0x3_handler, 0x3);
        map_irq_handler!(idt, irq_0x4_handler, 0x4);
        map_irq_handler!(idt, irq_0x5_handler, 0x5);
        map_irq_handler!(idt, irq_0x6_handler, 0x6);
        map_irq_handler!(idt, irq_0x7_handler, 0x7);
        map_irq_handler!(idt, irq_0x8_handler, 0x8);
        map_irq_handler!(idt, irq_0x9_handler, 0x9);
        map_irq_handler!(idt, irq_0xa_handler, 0xA);
        map_irq_handler!(idt, irq_0xb_handler, 0xB);
        map_irq_handler!(idt, irq_0xc_handler, 0xC);
        map_irq_handler!(idt, irq_0xd_handler, 0xD);
        map_irq_handler!(idt, irq_0xe_handler, 0xE);
        map_irq_handler!(idt, irq_0xf_handler, 0xF);

        idt
    };
}

///////////////////////////////
/// Interrupt Request (IRQ)
///////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IRQ {
    Timer = pics::M_OFFSET,
    Keyboard,
    RTC = pics::S_OFFSET,
}

impl IRQ {
    /// Default handler.
    fn default_handler() {}

    /// Converts index to pin.
    pub fn index_to_pin(idx: u8) -> u8 { pics::M_OFFSET + idx }

    /// Converts pin to index.
    pub fn pin_to_index(pin: IRQ) -> u8 { (pin as u8) - pics::M_OFFSET }
}

// Stamp out IRQ handlers.
generate_irq_handler!(irq_0x0_handler, 0x0);
generate_irq_handler!(irq_0x1_handler, 0x1);
generate_irq_handler!(irq_0x2_handler, 0x2);
generate_irq_handler!(irq_0x3_handler, 0x3);
generate_irq_handler!(irq_0x4_handler, 0x4);
generate_irq_handler!(irq_0x5_handler, 0x5);
generate_irq_handler!(irq_0x6_handler, 0x6);
generate_irq_handler!(irq_0x7_handler, 0x7);
generate_irq_handler!(irq_0x8_handler, 0x8);
generate_irq_handler!(irq_0x9_handler, 0x9);
generate_irq_handler!(irq_0xa_handler, 0xA);
generate_irq_handler!(irq_0xb_handler, 0xB);
generate_irq_handler!(irq_0xc_handler, 0xC);
generate_irq_handler!(irq_0xd_handler, 0xD);
generate_irq_handler!(irq_0xe_handler, 0xE);
generate_irq_handler!(irq_0xf_handler, 0xF);


/// Initializes the IDT.
pub(crate) fn init() {
    IDT.load();
    success!("IDT initialized");
}

/// Sets the interrupt handler for the given index.
pub(crate) fn set_irq_handler(pin: IRQ, handler: fn()) {
    instructions::interrupts::without_interrupts(
        || {
            let mut irq_handlers = IRQ_HANDLERS.lock();

            let idx = IRQ::pin_to_index(pin);
            irq_handlers[idx as usize] = handler;

            clear_interrupt_mask(idx);
        }
    );
}

/// Sets interrupt mask for the specified index.
#[allow(dead_code)]
fn set_interrupt_mask(idx: u8) {
    let (interrupt_line, port_addr) = if idx < pics::M_PIN_COUNT {
        (idx, pics::M_DATA_PORT)
    } else {
        (idx - pics::M_PIN_COUNT, pics::S_DATA_PORT)
    };

    let mut port = Port::new(port_addr);

    unsafe {
        let byte: u8 = port.read();
        port.write(byte | (1 << interrupt_line));
    }
}

/// Clears interrupt mask for the specified index.
fn clear_interrupt_mask(idx: u8) {
    let (interrupt_line, port_addr) = if idx < pics::M_PIN_COUNT {
        (idx, pics::M_DATA_PORT)
    } else {
        (idx - pics::M_PIN_COUNT, pics::S_DATA_PORT)
    };

    let mut port = Port::new(port_addr);

    unsafe {
        let byte: u8 = port.read();
        port.write(byte & !(1 << interrupt_line));
    }
}

/// A handler for breakpoint exceptions.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT");
    println!("{:#?}", stack_frame);
}

/// A handler for double fault exceptions.
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _err_code: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT");
    panic!("{:#?}", stack_frame);
}

/// A handler for page fault exceptions.
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, err_code: PageFaultErrorCode) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed address: {:?}", Cr2::read());
    println!("Error code: {:?}", err_code);
    println!("{:#?}", stack_frame);

    hlt_loop();
}
