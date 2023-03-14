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
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{hlt_loop, println, success};
use crate::kernel::gdt;

////////////////
// Attributes
////////////////

/// Offset of PIC 1.
const PIC_1_OFFSET: u8 = 32;
/// Pin count of PIC 1.
const PIC_1_PIN_COUNT: u8 = 8;
/// Data port for PIC 1.
const PIC_1_DATA_PORT: u16 = 0x21;

/// Offset of PIC 2.
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + PIC_1_PIN_COUNT;
/// Pin count of PIC 2.
const PIC_2_PIN_COUNT: u8 = 8;
/// Data port for PIC 2.
const PIC_2_DATA_PORT: u16 = 0xA1;

/// Total interrupt pins.
const TOTAL_INTERRUPT_PINS: u8 = PIC_1_PIN_COUNT + PIC_2_PIN_COUNT;

/////////////
// Mutexes
/////////////

/// 8259 Programmable Interrupt Controller (PIC)
///
/// The 8259 Programmable Interrupt Controller (PIC) is one of the most important chips making up
/// the x86 architecture. Without it, the x86 architecture would not be an interrupt driven
/// architecture. The function of the 8259A is to manage hardware interrupts and send them to the
/// appropriate system interrupt. This allows the system to respond to devices needs without loss
/// of time.
///
/// OS Dev Wiki: https://wiki.osdev.org/8259_PIC
pub(crate) static PICS: Mutex<ChainedPics> = Mutex::new(
    unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }
);

///////////////////////
/// Interrupt Index
///////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    RTC = PIC_2_OFFSET,
}

/// Calculates the interrupt index.
fn calc_interrupt_index(interrupt: u8) -> u8 {
    PIC_1_OFFSET + interrupt
}

/// Default interrupt handler.
fn default_interrupt_handler() {}

lazy_static! {
    /// A global interface for interrupt handlers.
    static ref INTERRUPT_HANDLERS: Mutex<[fn(); TOTAL_INTERRUPT_PINS as usize]> = Mutex::new(
        [default_interrupt_handler; TOTAL_INTERRUPT_PINS as usize]
    );
}

/// Generates the interrupt handler.
macro_rules! generate_interrupt_handler {
    ($handler:ident, $interrupt:expr) => {
        extern "x86-interrupt" fn $handler(_stack_frame: InterruptStackFrame) {
            let interrupt_handlers = INTERRUPT_HANDLERS.lock();
            interrupt_handlers[$interrupt]();
            unsafe { PICS.lock().notify_end_of_interrupt(calc_interrupt_index($interrupt)); }
        }
    };
}

// Stamp out interrupt handlers.
generate_interrupt_handler!(interrupt_0x0_handler, 0x0);
generate_interrupt_handler!(interrupt_0x1_handler, 0x1);
generate_interrupt_handler!(interrupt_0x2_handler, 0x2);
generate_interrupt_handler!(interrupt_0x3_handler, 0x3);
generate_interrupt_handler!(interrupt_0x4_handler, 0x4);
generate_interrupt_handler!(interrupt_0x5_handler, 0x5);
generate_interrupt_handler!(interrupt_0x6_handler, 0x6);
generate_interrupt_handler!(interrupt_0x7_handler, 0x7);
generate_interrupt_handler!(interrupt_0x8_handler, 0x8);
generate_interrupt_handler!(interrupt_0x9_handler, 0x9);
generate_interrupt_handler!(interrupt_0xa_handler, 0xA);
generate_interrupt_handler!(interrupt_0xb_handler, 0xB);
generate_interrupt_handler!(interrupt_0xc_handler, 0xC);
generate_interrupt_handler!(interrupt_0xd_handler, 0xD);
generate_interrupt_handler!(interrupt_0xe_handler, 0xE);
generate_interrupt_handler!(interrupt_0xf_handler, 0xF);

/// Maps the interrupt handler.
macro_rules! map_interrupt_handler {
    ($reference:ident, $handler:ident, $interrupt:expr) => {
        $reference[calc_interrupt_index($interrupt) as usize].set_handler_fn($handler);
    };
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
        map_interrupt_handler!(idt, interrupt_0x0_handler, 0x0);
        map_interrupt_handler!(idt, interrupt_0x1_handler, 0x1);
        map_interrupt_handler!(idt, interrupt_0x2_handler, 0x2);
        map_interrupt_handler!(idt, interrupt_0x3_handler, 0x3);
        map_interrupt_handler!(idt, interrupt_0x4_handler, 0x4);
        map_interrupt_handler!(idt, interrupt_0x5_handler, 0x5);
        map_interrupt_handler!(idt, interrupt_0x6_handler, 0x6);
        map_interrupt_handler!(idt, interrupt_0x7_handler, 0x7);
        map_interrupt_handler!(idt, interrupt_0x8_handler, 0x8);
        map_interrupt_handler!(idt, interrupt_0x9_handler, 0x9);
        map_interrupt_handler!(idt, interrupt_0xa_handler, 0xA);
        map_interrupt_handler!(idt, interrupt_0xb_handler, 0xB);
        map_interrupt_handler!(idt, interrupt_0xc_handler, 0xC);
        map_interrupt_handler!(idt, interrupt_0xd_handler, 0xD);
        map_interrupt_handler!(idt, interrupt_0xe_handler, 0xE);
        map_interrupt_handler!(idt, interrupt_0xf_handler, 0xF);

        idt
    };
}

///////////////
// Utilities //
///////////////

/// Initializes the IDT and PICs.
pub(crate) fn init() {
    init_idt();
    unsafe {
        init_pics();
    }
}

/// Initializes the IDT.
fn init_idt() {
    IDT.load();
    success!("IDT initialized");
}

/// Initializes the PICs.
unsafe fn init_pics() {
    PICS.lock().initialize();
    success!("PICs initialized");
}

/// Enables interrupts.
pub(crate) fn enable() {
    instructions::interrupts::enable();
    success!("Interrupts enabled");
}

/// Sets the interrupt handler for the given index.
pub(crate) fn set_interrupt_handler(index: InterruptIndex, handler: fn()) {
    instructions::interrupts::without_interrupts(
        || {
            let mut interrupt_handlers = INTERRUPT_HANDLERS.lock();
            let real_index = (index as u8) - PIC_1_OFFSET;
            interrupt_handlers[real_index as usize] = handler;

            clear_interrupt_mask(real_index);
        }
    );
}

/// Sets interrupt mask for the specified index.
#[allow(dead_code)]
fn set_interrupt_mask(index: u8) {
    let mut port = Port::new(if index < PIC_1_PIN_COUNT { PIC_1_DATA_PORT } else { PIC_2_DATA_PORT });

    let interrupt_line = if index < PIC_1_PIN_COUNT { index } else { index - PIC_1_PIN_COUNT };
    unsafe {
        let value: u8 = port.read() | (1 << interrupt_line);
        port.write(value);
    }
}

/// Clears interrupt mask for the specified index.
fn clear_interrupt_mask(index: u8) {
    let mut port = Port::new(if index < PIC_1_PIN_COUNT { PIC_1_DATA_PORT } else { PIC_2_DATA_PORT });

    let interrupt_line = if index < PIC_1_PIN_COUNT { index } else { index - PIC_1_PIN_COUNT };
    unsafe {
        let value: u8 = port.read() & !(1 << interrupt_line);
        port.write(value);
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
