use acpi::platform::interrupt::Apic;
use x86::msr::APIC_BASE;
use x86_64::PhysAddr;
use x86_64::registers::model_specific::Msr;

use crate::kernel::memory;
use crate::omneity;

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

unsafe fn read(base: usize, register: usize) -> u32 {
    let tgt = base + register;
    let tgt = tgt as *mut u32;
    core::ptr::read_volatile(tgt)
}

unsafe fn write(base: usize, register: usize, value: u32) {
    let tgt = base + register;
    let tgt = tgt as *mut u32;
    core::ptr::write_volatile(tgt, value);
}

unsafe fn get_id(base: usize) -> u32
{
    read(base, LAPIC_ID) >> 24
}

pub unsafe fn init(apic: &Apic) {
    let mut msr = Msr::new(APIC_BASE);
    let cur = msr.read();
    msr.write(cur | 0x800); // Set bit 11.
    let cur = msr.read();

    let apic_base_addr = memory::phys_to_virt_addr(PhysAddr::new(apic.local_apic_address));
    let base = apic_base_addr.as_u64() as usize;

    // spurious vectors.
    write(base, LAPIC_SVR, 0x100 | 0xFF); // enable or disable apic.
}
