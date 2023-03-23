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

use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};

use acpi::AcpiError;
use acpi::fadt::Fadt;

///////////////////
// Cached Values
///////////////////

/// Cached `ACPI Enable` register value.
static ACPI_ENABLE: AtomicU8 = AtomicU8::new(u8::MAX);
/// Cached `ACPI Disable` register value.
static ACPI_DISABLE: AtomicU8 = AtomicU8::new(u8::MAX);
/// Cached `PM-1A Control Block` register value.
static PM1A_CTRL_BLK_PTR: AtomicU64 = AtomicU64::new(u64::MAX);

///////////////
// Utilities
///////////////

/// Reads and caches necessary registers.
pub(super) fn read(sdt: &Fadt) -> Result<(), AcpiError> {
    ACPI_ENABLE.store(sdt.acpi_enable, Ordering::Relaxed);
    ACPI_DISABLE.store(sdt.acpi_disable, Ordering::Relaxed);
    PM1A_CTRL_BLK_PTR.store(sdt.pm1a_control_block()?.address, Ordering::Relaxed);

    Ok(())
}

/// Returns the `ACPI Enable` register value.
pub fn acpi_enable() -> u8 { ACPI_ENABLE.load(Ordering::Relaxed) }

/// Returns the `ACPI Disable` register value.
pub fn acpi_disable() -> u8 { ACPI_DISABLE.load(Ordering::Relaxed) }

/// Returns the `PM-1A Control Block` register value.
pub fn pm1a_ctrl_blk_ptr() -> u64 { PM1A_CTRL_BLK_PTR.load(Ordering::Relaxed) }
