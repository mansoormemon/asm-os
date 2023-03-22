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

use core::ptr::NonNull;

use acpi::{AcpiTables, PhysicalMapping};
use acpi::AcpiHandler;
use x86_64::PhysAddr;

use crate::{failure, success, warning};
use crate::krnl::memory;

pub mod dsdt;
pub mod fadt;
pub mod madt;

/////////////////
// ACPI Tables
/////////////////

/// Differentiated System Description Table (DSDT).
pub const DSDT: &str = "DSDT";
/// Fixed ACPI Description Table (FADT).
pub const FADT: &str = "FACP";
/// Multiple APIC Description Table (MADT).
pub const MADT: &str = "APIC";
/// Root System Description Pointer (RSDP).
pub const RSDP: &str = "RSD PTR";
/// Extended System Description Table (XSDT).
pub const XSDT: &str = "XSDT";

///////////////
// Utilities
///////////////

/// Initializes the ACPI and stores required parameters.
pub(crate) fn init() {
    if let Ok(acpi) = unsafe { AcpiTables::search_for_rsdp_bios(CustomACPIHandler) } {
        for (sign, sdt) in acpi.sdts {
            match sign.as_str() {
                FADT => fadt::parse(sdt),
                MADT => madt::parse(sdt),
                _ => {}
            }
        }

        if let Some(sdt) = acpi.dsdt {
            dsdt::parse(sdt);
        } else {
            warning!("ACPI: D={:?}", CustomACPIError::InvalidDSDT);
        }

        success!("ACPI initialized");
    } else {
        failure!("E={:?}", CustomACPIError::ACPINotFound);
    };
}

/// Converts the given physical address to virtual address and returns it.
fn read_addr<T>(phys_addr: usize) -> T where T: Copy {
    let virt_addr = memory::phys_to_virt_addr(PhysAddr::new(phys_addr as u64));
    unsafe { *virt_addr.as_ptr::<T>() }
}

///////////////////////////
/// Custom ACPI Handler
///////////////////////////
#[derive(Clone)]
struct CustomACPIHandler;

impl AcpiHandler for CustomACPIHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let virt_addr = memory::phys_to_virt_addr(PhysAddr::new(physical_address as u64));
        PhysicalMapping::new(physical_address, NonNull::new(virt_addr.as_mut_ptr()).unwrap(), size, size, Self)
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

/////////////////////////
/// Custom ACPI Error
////////////////////////
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CustomACPIError {
    ACPINotFound = 0x0,
    AMLParseFailure = 0x1,
    InvalidDSDT = 0x2,
    FADTNotFound = 0x3,
    S5ParseFailure = 0x4,
}
