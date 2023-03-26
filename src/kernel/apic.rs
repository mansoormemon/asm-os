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

pub mod io;
pub mod local;


pub(crate) fn init() -> Result<(), ()> {
    unsafe { pics::PIC_8259.lock().disable() };

    let apic = acpi::madt::get_interrupt_model().unwrap();
    let proc_info = acpi::madt::get_processor_info().unwrap();

    pub const APIC_BASE: u32 = 0x1B;

    match apic {
        InterruptModel::Apic(apic) => unsafe {
            local::init(&apic);
            io::init(&apic);

            // local_apic_out(base, LAPIC_TPR, 0);
            //
            // local_apic_out(base, LAPIC_TDCR, 0x3);
            // local_apic_out(base, LAPIC_TICR, u32::MAX);
            // // pit::sleep(0.01);
            // local_apic_out(base, LAPIC_TIMER, 0);
            //
            // let TMR_PERIODIC = 0x20000;
            //
            // let ticks_in_10_ms = 0xFFFFFFFF - local_apic_in(base, LAPIC_TCCR);
            // local_apic_out(base, LAPIC_TIMER, 32 | TMR_PERIODIC);
            // local_apic_out(base, LAPIC_TDCR, 0x3);
            // local_apic_out(base, LAPIC_TICR, ticks_in_10_ms);
            //
            // for (io_apic, iso) in apic.io_apics.iter().zip(apic.interrupt_source_overrides.iter()) {
            //     let address = memory::phys_to_virt_addr(PhysAddr::new(io_apic.address as u64));
            //     let address = address.as_u64();
            //     let mut ioapic = x86::apic::ioapic::IoApic::new(address as usize);
            //     omneity!("{:?}", (
            //         ioapic.id(),
            //         ioapic.version(),
            //         ioapic.supported_interrupts(),
            //     ));
            //
            //     for i in 0..ioapic.supported_interrupts() {
            //         let val: u64 = (1u64 << 16) | (0x20u64 + i as u64);
            //         io_apic_set_entry(address as usize, i, val);
            //     }
            //
            //
            //     // Get number of entries supported by the IO APIC
            //     let x: u32 = io_apic_in(address as usize, IOAPICVER as u8);
            //     let count: u32 = ((x >> 16) & 0xff) + 1;
            //     // maximum redirection entry
            //     omneity!("I/O APIC pins = {}", count);
            //
            //     let isr = 1; // keyboard
            //     let mut entry = IOREDTBL + (isr * 2) as usize;
            //     let mut entry_val = io_apic_in(address as usize, entry as u8);
            //     omneity!("{:b}", entry_val);
            //     entry_val &= !(1 << 16);
            //     omneity!("{:b}", entry_val);
            //     entry &= !0x700;
            //     omneity!("{:b}", entry);
            //     entry &= !0x800;
            //     omneity!("{:b}", entry);
            //     entry = entry | ((local_apic_get_id(base) as usize) << 56);
            //     omneity!("{:b}", entry);
            //     io_apic_set_entry(address as usize, isr as u8, entry as u64);
        }
        _ => {}
    }

    Ok(())
}

fn interrupt_hander_apic_timer() {
    pit::timer_irq_handler();
}
