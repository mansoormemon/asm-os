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

use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU8, Ordering};

use acpi::Sdt;

///////////////////
// Cached Values
///////////////////

/// Value of SCI interrupt in the FADT register.
pub(super) static SCI_INTERRUPT: AtomicU16 = AtomicU16::new(u16::MAX);
/// Value of SMI command port in the FADT register.
pub(super) static SMI_COMMAND_PORT: AtomicU32 = AtomicU32::new(u32::MAX);
/// Value of ACPI enable in the FADT register.
pub(super) static ACPI_ENABLE: AtomicU8 = AtomicU8::new(u8::MAX);
/// Value of ACPI disable in the FADT register.
pub(super) static ACPI_DISABLE: AtomicU8 = AtomicU8::new(u8::MAX);
/// Value of PM-1A control block in the FADT register.
pub(crate) static PM_1A_CONTROL_BLOCK: AtomicU32 = AtomicU32::new(u32::MAX);
/// Value of PM-1B control block in the FADT register.
pub(super) static PM_1B_CONTROL_BLOCK: AtomicU32 = AtomicU32::new(u32::MAX);

///////////////////////////////////////////
/// Fixed ACPI Description Table (FADT)
///////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    PreferredPowerManagementProfile = 45,
    SCIInterrupt = 46,
    SMICommandPort = 48,
    ACPIEnable = 52,
    ACPIDisable = 53,
    S4BIOSREQ = 54,
    PStateControl = 55,
    PM1AEventBlock = 56,
    PM1BEventBlock = 60,
    PM1AControlBlock = 64,
    PM1BControlBlock = 68,
    PM2ControlBlock = 72,
    PMTimerBlock = 76,
    GPE0Block = 80,
    GPE1Block = 84,
    PM1EventLength = 88,
    PM1ControlLength = 89,
    PM2ControlLength = 90,
    PMTimerLength = 91,
    GPE0Length = 92,
    GPE1Length = 93,
    GPE1Base = 94,
    CStateControl = 95,
    WorstC2Latency = 96,
    WorstC3Latency = 98,
    FlushSize = 100,
    FlushStride = 102,
    DutyOffset = 103,
    DutyWidth = 104,
    DayAlarm = 105,
    MonthAlarm = 106,
    Century = 107,
}

pub(super) fn parse(sdt: Sdt) {
    SCI_INTERRUPT.store(
        read_fadt(sdt.physical_address, Register::SCIInterrupt),
        Ordering::Relaxed,
    );
    SMI_COMMAND_PORT.store(
        read_fadt(sdt.physical_address, Register::SMICommandPort),
        Ordering::Relaxed,
    );
    ACPI_ENABLE.store(
        read_fadt(sdt.physical_address, Register::ACPIEnable),
        Ordering::Relaxed,
    );
    ACPI_DISABLE.store(
        read_fadt(sdt.physical_address, Register::ACPIDisable),
        Ordering::Relaxed,
    );
    PM_1A_CONTROL_BLOCK.store(
        read_fadt(sdt.physical_address, Register::PM1AControlBlock),
        Ordering::Relaxed,
    );
    PM_1B_CONTROL_BLOCK.store(
        read_fadt(sdt.physical_address, Register::PM1BControlBlock),
        Ordering::Relaxed,
    );
}

/// Reads the value of the given register and returns it.
fn read_fadt<T>(fadt_phys_addr: usize, register: Register) -> T where T: Copy {
    super::read_addr(fadt_phys_addr + register as usize)
}
