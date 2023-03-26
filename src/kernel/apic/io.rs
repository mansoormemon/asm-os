use core::fmt;
use core::fmt::{Formatter, LowerHex};
use acpi::platform::interrupt::Apic;
use bitflags::bitflags;
use x86_64::PhysAddr;

use crate::kernel::memory;
use crate::omneity;

macro_rules! define {
    ($name:ident, $val:expr) => {
        pub const $name: usize = $val;
    };
}

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

define!(IOREGSEL,  0x00);
define!(IOWIN,     0x10);

// IO APIC Registers
define!(IOAPICID,  0x00);
define!(IOAPICVER, 0x01);
define!(IOAPICARB, 0x02);
define!(IOREDTBL,  0x10);

pub const IRQ_MODE_MASK: u32 = 0x0000_0700;

/// IOAPIC interrupt modes.
#[derive(Debug)]
#[repr(u8)]
pub enum IrqMode {
    /// Asserts the INTR signal on all allowed processors.
    Fixed = 0b000,
    /// Asserts the INTR signal on the lowest priority processor allowed.
    LowestPriority = 0b001,
    /// System management interrupt.
    /// Requires edge-triggering.
    SystemManagement = 0b010,
    /// Asserts the NMI signal on all allowed processors.
    /// Requires edge-triggering.
    NonMaskable = 0b100,
    /// Asserts the INIT signal on all allowed processors.
    /// Requires edge-triggering.
    Init = 0b101,
    /// Asserts the INTR signal as a signal that originated in an
    /// externally-connected interrupt controller.
    /// Requires edge-triggering.
    External = 0b111,
}

impl IrqMode {
    pub(super) fn as_u32(self) -> u32 {
        (self as u32) << 8
    }
}

impl TryFrom<u32> for IrqMode {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match (value & IRQ_MODE_MASK) >> 8 {
            0b000 => Ok(IrqMode::Fixed),
            0b001 => Ok(IrqMode::LowestPriority),
            0b010 => Ok(IrqMode::SystemManagement),
            0b100 => Ok(IrqMode::NonMaskable),
            0b101 => Ok(IrqMode::Init),
            0b111 => Ok(IrqMode::External),
            other => Err(other),
        }
    }
}

bitflags! {
    /// Redirection table entry flags.
    pub struct IrqFlags: u32 {
        /// Logical destination mode (vs physical)
        const LOGICAL_DEST = 1 << 11;
        /// Delivery status: send pending (vs idle, readonly)
        const SEND_PENDING = 1 << 12;
        /// Low-polarity interrupt signal (vs high-polarity)
        const LOW_ACTIVE = 1 << 13;
        /// Remote IRR (readonly)
        const REMOTE_IRR = 1 << 14;
        /// Level-triggered interrupt (vs edge-triggered)
        const LEVEL_TRIGGERED = 1 << 15;
        /// Masked interrupt (vs unmasked)
        const MASKED = 1 << 16;
    }
}

#[derive(Default)]
pub struct RedirectionTableEntry {
    low: u32,
    high: u32,
}

impl RedirectionTableEntry {
    pub(crate) fn from_raw(low: u32, high: u32) -> Self {
        Self { low, high }
    }

    pub(crate) fn into_raw(self) -> (u32, u32) {
        (self.low, self.high)
    }

    /// Returns the interrupt vector.
    pub fn vector(&self) -> u8 {
        (self.low & 0xff) as u8
    }

    /// Sets the interrupt vector to `vector`.
    pub fn set_vector(&mut self, vector: u8) {
        self.low = self.low & !0xff | vector as u32
    }

    /// Returns the interrupt delivery mode.
    pub fn mode(&self) -> IrqMode {
        self.low.try_into().unwrap()
    }

    /// Sets the interrupt delivery mode to `mode`.
    pub fn set_mode(&mut self, mode: IrqMode) {
        self.low = self.low & !IRQ_MODE_MASK | mode.as_u32()
    }

    pub fn flags(&self) -> IrqFlags {
        IrqFlags::from_bits_truncate(self.low)
    }

    /// Sets the redirection table entry flags to `flags`.
    pub fn set_flags(&mut self, flags: IrqFlags) {
        let ro_flags = IrqFlags::SEND_PENDING | IrqFlags::REMOTE_IRR;
        self.low = self.low & !(IrqFlags::all() - ro_flags).bits()
            | (flags - ro_flags).bits()
    }

    /// Returns the destination field.
    pub fn dest(&self) -> u8 {
        (self.high >> 24) as u8
    }

    /// Sets the destination field to `dest`.
    pub fn set_dest(&mut self, dest: u8) {
        self.high = (dest as u32) << 24;
    }
}

// Gets the lower segment selector for `irq`
pub fn lo(irq: u8) -> u32 {
    (IOREDTBL + (2 * usize::from(irq))) as u32
}

// Gets the upper segment selector for `irq`
pub fn hi(irq: u8) -> u32 {
    lo(irq) + 1
}

impl fmt::Debug for RedirectionTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RedirectionTableEntry")
         .field("vector", &self.vector())
         .field("mode", &self.mode())
         .field("flags", &self.flags())
         .field("dest", &self.dest())
         .finish()
    }
}

unsafe fn read(base: usize, reg: u8) -> u32 {
    let tgt_io_reg_sel = base + IOREGSEL;
    let tgt_io_reg_sel = tgt_io_reg_sel as *mut u32;
    core::ptr::write_volatile(tgt_io_reg_sel, reg as u32);

    let tgt_io_win = base + IOWIN;
    let tgt_io_win = tgt_io_win as *mut u32;
    core::ptr::read_volatile(tgt_io_win)
}

unsafe fn write(base: usize, reg: u8, value: u32) {
    let tgt_io_reg_sel = base + IOREGSEL;
    let tgt_io_reg_sel = tgt_io_reg_sel as *mut u32;
    core::ptr::write_volatile(tgt_io_reg_sel, reg as u32);

    let tgt_io_win = base + IOWIN;
    let tgt_io_win = tgt_io_win as *mut u32;
    core::ptr::write_volatile(tgt_io_win, value);
}

unsafe fn io_apic_set_entry(base: usize, index: u8, data: u64) {
    write(base, (IOREDTBL + (index as usize) * 2) as u8, data as u32);
    write(base, (IOREDTBL + (index as usize) * 2 + 1) as u8, (data >> 32) as u32);
}

pub unsafe fn init(apic: &Apic) {
    for io_apic in apic.io_apics.iter() {
        let base = memory::phys_to_virt_addr(PhysAddr::new(io_apic.address as u64));
        let base = base.as_u64();

        let irq = 1;
        let mut reg = RedirectionTableEntry::default();

        reg.set_vector(33);

        omneity!("{:?}", reg);

        let (low, high) = reg.into_raw();

        write(base as usize, lo(irq) as u8, low);
        write(base as usize, hi(irq) as u8, high);
    }
}
