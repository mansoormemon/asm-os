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

use core::ptr::NonNull;

use acpi::{AcpiError, AcpiTables, PhysicalMapping};
use acpi::AcpiHandler;
use acpi::fadt::Fadt;
use acpi::madt::Madt;
use acpi::sdt::Signature;
use aml::AmlError;
use x86_64::PhysAddr;

use crate::kernel::memory;

pub mod dsdt;
pub mod fadt;
pub mod madt;

///////////////
// Utilities
///////////////

/// Initializes the ACPI and stores required parameters.
pub(crate) fn init() -> Result<(), GenericError> {
    let acpi = unsafe { AcpiTables::search_for_rsdp_bios(CustomACPIHandler) }?;

    let fadt = unsafe { acpi.get_sdt::<Fadt>(Signature::FADT) }?.ok_or(AcpiError::TableMissing(Signature::FADT))?;
    fadt::read(&fadt)?;

    let dsdt = acpi.dsdt.as_ref().ok_or(AcpiError::TableMissing(Signature::DSDT))?;
    dsdt::read(&dsdt)?;

    let madt = unsafe { acpi.get_sdt::<Madt>(Signature::MADT) }?.ok_or(AcpiError::TableMissing(Signature::MADT))?;
    madt::read(&madt).unwrap();

    Ok(())
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

/////////////////////
/// Generic Error
/////////////////////
#[derive(Debug)]
pub enum GenericError {
    ACPI(AcpiError),
    AML(AmlError),
}

impl From<AcpiError> for GenericError {
    fn from(value: AcpiError) -> Self { Self::ACPI(value) }
}

impl From<AmlError> for GenericError {
    fn from(value: AmlError) -> Self { Self::AML(value) }
}
