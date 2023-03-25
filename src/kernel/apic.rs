// MIT License
//
// Copyright (c) 2023 Mansoor Ahmed Memon.
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

use core::arch::asm;

use ::acpi::InterruptModel;
use x86_64::instructions::port::Port;
use x86_64::PhysAddr;
use x86_64::registers::model_specific::Msr;

use crate::{omneity, print, println};
use crate::kernel::{acpi, idt, memory, pics, pit};

macro_rules! define {
($name:ident, $val:expr) => {
pub const $name: usize = $val;
};
}

define!(LAPIC_ID, 0x0020);// Local APIC ID
define!(LAPIC_VER, 0x0030); // Local APIC Version
define!(LAPIC_TPR, 0x0080); // Task Priority
define!(LAPIC_APR, 0x0090); // Arbitration Priority
define!(LAPIC_PPR, 0x00a0); // Processor Priority
define!(LAPIC_EOI, 0x00b0); // EOI
define!(LAPIC_RRD, 0x00c0); // Remote Read
define!(LAPIC_LDR, 0x00d0); // Logical Destination
define!(LAPIC_DFR, 0x00e0); // Destination Format
define!(LAPIC_SVR, 0x00f0); // Spurious Interrupt Vector
define!(LAPIC_ISR, 0x0100); // In-Service (8 registers)
define!(LAPIC_TMR, 0x0180);// Trigger Mode (8 registers)
define!(LAPIC_IRR, 0x0200);// Interrupt Request (8 registers)
define!(LAPIC_ESR, 0x0280);// Error Status
define!(LAPIC_ICRLO, 0x0300);// Interrupt Command
define!(LAPIC_ICRHI, 0x0310);// Interrupt Command [63:32]
define!(LAPIC_TIMER, 0x0320);// LVT Timer
define!(LAPIC_THERMAL, 0x0330);// LVT Thermal Sensor
define!(LAPIC_PERF, 0x0340);// LVT Performance Counter
define!(LAPIC_LINT0, 0x0350);// LVT LINT0
define!(LAPIC_LINT1, 0x0360);// LVT LINT1
define!(LAPIC_ERROR, 0x0370);// LVT Error
define!(LAPIC_TICR, 0x0380);// Initial Count (for Timer)
define!(LAPIC_TCCR, 0x0390);// Current Count (for Timer)
define!(LAPIC_TDCR, 0x03e0);// Divide Configuration (for Timer)

// Delivery Mode
define!(ICR_FIXED, 0x00000000);
define!(ICR_LOWEST, 0x00000100);
define!(ICR_SMI, 0x00000200);
define!(ICR_NMI, 0x00000400);
define!(ICR_INIT, 0x00000500);
define!(ICR_STARTUP, 0x00000600);

// Destination Mode
define!(ICR_PHYSICAL, 0x00000000);
define!(ICR_LOGICAL, 0x00000800);

// Delivery Status
define!(ICR_IDLE, 0x00000000);
define!(ICR_SEND_PENDING, 0x00001000);

// Level
define!(ICR_DEASSERT, 0x00000000);
define!(ICR_ASSERT, 0x00004000);

// Trigger Mode
define!(ICR_EDGE, 0x00000000);
define!(ICR_LEVEL, 0x00008000);

// Destination Shorthand
define!(ICR_NO_SHORTHAND, 0x00000000);
define!(ICR_SELF, 0x00040000);
define!(ICR_ALL_INCLUDING_SELF, 0x00080000);
define!(ICR_ALL_EXCLUDING_SELF, 0x000c0000);

// Destination Field
define!(ICR_DESTINATION_SHIFT, 24);


unsafe fn local_apic_in(base: usize, register: usize) -> u32 {
    let tgt = base + register;
    let tgt = tgt as *mut u32;
    core::ptr::read_volatile(tgt)
}

unsafe fn local_apic_out(base: usize, register: usize, value: u32) {
    let tgt = base + register;
    let tgt = tgt as *mut u32;
    core::ptr::write_volatile(tgt, value);
}

unsafe fn local_apic_get_id(base: usize) -> u32
{
    local_apic_in(base, LAPIC_ID) >> 24
}

define!(IOREGSEL,  0x00);
define!(IOWIN,     0x10);

// ------------------------------------------------------------------------------------------------
// IO APIC Registers
define!(IOAPICID,  0x00);
define!(IOAPICVER, 0x01);
define!(IOAPICARB, 0x02);
define!(IOREDTBL,  0x10);

unsafe fn io_apic_out(base: usize, reg: u8, value: u32) {
    let tgt_io_reg_sel = base + IOREGSEL;
    let tgt_io_reg_sel = tgt_io_reg_sel as *mut u32;
    core::ptr::write_volatile(tgt_io_reg_sel, reg as u32);

    let tgt_io_win = base + IOWIN;
    let tgt_io_win = tgt_io_win as *mut u32;
    core::ptr::write_volatile(tgt_io_win, value);
}

unsafe fn io_apic_in(base: usize, reg: u8) -> u32 {
    let tgt_io_reg_sel = base + IOREGSEL;
    let tgt_io_reg_sel = tgt_io_reg_sel as *mut u32;
    core::ptr::write_volatile(tgt_io_reg_sel, reg as u32);

    let tgt_io_win = base + IOWIN;
    let tgt_io_win = tgt_io_win as *mut u32;
    core::ptr::read_volatile(tgt_io_win)
}

unsafe fn io_apic_set_entry(base: usize, index: u8, data: u64) {
    io_apic_out(base, (IOREDTBL + (index as usize) * 2) as u8, data as u32);
    io_apic_out(base, (IOREDTBL + (index as usize) * 2 + 1) as u8, (data >> 32) as u32);
}


pub(crate) fn init() -> Result<(), ()> {
    let apic = acpi::madt::get_interrupt_model().unwrap();
    let proc_info = acpi::madt::get_processor_info().unwrap();

    pub const APIC_BASE: u32 = 0x1B;

    match apic {
        InterruptModel::Apic(apic) => unsafe {
            let mut msr = Msr::new(APIC_BASE);
            let cur = msr.read();
            println!("{:#x}", cur);
            msr.write(cur | 0x800); // Set bit 11.
            let cur = msr.read();
            println!("{:#x}", cur);

            let apic_base_addr = memory::phys_to_virt_addr(PhysAddr::new(apic.local_apic_address));
            let base = apic_base_addr.as_u64() as usize;

            local_apic_out(base, LAPIC_DFR, 0xffffffff); // Flat mode
            local_apic_out(base, LAPIC_LDR, 0x01000000); // All cpus use logical id 1
            // Configure Spurious Interrupt Vector Register
            local_apic_out(base, LAPIC_SVR, 0x100 | 0xFF);

            local_apic_out(base, LAPIC_TPR, 0);

            local_apic_out(base, LAPIC_TDCR, 0x3);
            local_apic_out(base, LAPIC_TICR, 0xFFFFFFFF);
            // pit::sleep(0.01);
            local_apic_out(base, LAPIC_TIMER, 0);

            println!("{:?}", pics::PIC_8259.lock().read_masks());
            let TMR_PERIODIC = 0x20000;

            let ticks_in_10_ms = 0xFFFFFFFF - local_apic_in(base, LAPIC_TCCR);
            local_apic_out(base, LAPIC_TIMER, 32 | TMR_PERIODIC);
            local_apic_out(base, LAPIC_TDCR, 0x3);
            local_apic_out(base, LAPIC_TICR, ticks_in_10_ms);
            pics::PIC_8259.lock().disable();

            for (io_apic, iso) in apic.io_apics.iter().zip(apic.interrupt_source_overrides.iter()) {
                let address = memory::phys_to_virt_addr(PhysAddr::new(io_apic.address as u64));
                let address = address.as_u64();
                let mut ioapic = x86::apic::ioapic::IoApic::new(address as usize);
                omneity!("{:?}", (
                    ioapic.id(),
                    ioapic.version(),
                    ioapic.supported_interrupts(),
                ));
                for i in 0..ioapic.supported_interrupts() {
                    ioapic.enable(0, 0);
                }
                if iso.isa_source != 1 {
                    // Get number of entries supported by the IO APIC
                    let x: u32 = io_apic_in(address as usize, IOAPICVER as u8);
                    let count: u32 = ((x >> 16) & 0xff) + 1;
                    // maximum redirection entry
                    omneity!("I/O APIC pins = {}", count);

                    // Disable all entries
                    for i in 0..count {
                        // io_apic_set_entry(address as usize, i as u8, 1 << 16);

                        let entry = IOREDTBL + (i * 2) as usize;

                        io_apic_set_entry(address as usize, i as u8, (0x20 + i) as u64);
                    }

                    let isr = 1; // keyboard
                    let mut entry = IOREDTBL + (isr * 2) as usize;
                    omneity!("{:b}", entry);
                    entry &= !0x700;
                    omneity!("{:b}", entry);
                    entry &= !0x800;
                    omneity!("{:b}", entry);
                    entry |= ((local_apic_get_id(base) as usize) << 56);
                    omneity!("{:b}", entry);
                    // io_apic_set_entry(address as usize, isr as u8, entry as u64);
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn interrupt_hander_apic_timer() {
    pit::timer_irq_handler();
}