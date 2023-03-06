use instructions::port::Port;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{hlt_loop, println};
use crate::kernel::{gdt, task};

/// 8259 PIC
///
/// The 8259 Programmable Interrupt Controller (PIC) is one of the most important chips making up
/// the x86 architecture. Without it, the x86 architecture would not be an interrupt driven
/// architecture. The function of the 8259A is to manage hardware interrupts and send them to the
/// appropriate system interrupt. This allows the system to respond to devices needs without loss
/// of time.
///
/// OS Dev Wiki: https://wiki.osdev.org/8259_PIC

/// Offset of PIC 1 (Master).
pub const PIC_1_OFFSET: u8 = 32;

/// Offset of PIC 2 (Slave).
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// A global interface for PIC 1 (Master).
pub static PICS: Mutex<ChainedPics> = Mutex::new(
    unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }
);

/// Interrupt Index.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

// Interrupt Descriptor Table (IDT)
//
// The Interrupt Descriptor Table (IDT) is a data structure used by the x86 architecture to
// implement an interrupt vector table. The IDT is used by the processor to determine the correct
// response to interrupts and exceptions.
//
// Wikipedia: https://en.wikipedia.org/wiki/Interrupt_descriptor_table

lazy_static! {
    /// A global interface for Interrupt Descriptor Table (IDT).
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

/// Initializes the IDT.
pub fn init_idt() {
    IDT.load();
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

/// An interrupt handler for timer interrupts.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8); }
}

/// An interrupt handler for keyboard interrupts.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    const PORT_ADDR: u16 = 0x60;

    let mut port = Port::new(PORT_ADDR);
    let scancode: u8 = unsafe { port.read() };

    // Add the scancode to the waiting queue.
    task::keyboard::add_scancode(scancode);

    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard as u8); }
}
