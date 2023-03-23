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

///////////////////
// Cached Values
///////////////////

use alloc::boxed::Box;
use core::slice;
use core::sync::atomic::{AtomicU16, Ordering};

use acpi::AmlTable;
use aml::{AmlContext, AmlError, AmlName, AmlValue, DebugVerbosity};
use aml::Handler;
use x86_64::PhysAddr;

use crate::kernel::memory;

///////////////
// Constants
///////////////

/// Value of SLP_EN.
pub const SLP_EN: u16 = 0x2000;

/// Block code for S5.
pub const BLOCK_CODE_S5: &'static str = "\\_S5";

///////////////////
// Cached Values
///////////////////

/// Value of SLP_TYP_A from the AML tables.
static SLP_TYP_A: AtomicU16 = AtomicU16::new(u16::MAX);
/// Value of SLP_TYP_B from the AML tables.
static SLP_TYP_B: AtomicU16 = AtomicU16::new(u16::MAX);

////////////////
/// Block S5
////////////////
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
enum BlockS5 {
    SlpTypA,
    SlpTypB,
}

///////////////
// Utilities
///////////////

pub(super) fn read(sdt: &AmlTable) -> Result<(), AmlError> {
    let mut aml = AmlContext::new(Box::new(CustomAMLHandler), DebugVerbosity::None);

    let address = memory::phys_to_virt_addr(PhysAddr::new(sdt.address as u64));

    // Create AML table from raw parts.
    let stream = unsafe { slice::from_raw_parts(address.as_ptr(), sdt.length as usize) };
    aml.parse_table(stream)?;

    // Parse S5 block code from the AML table.
    let name = AmlName::from_str(BLOCK_CODE_S5)?;
    if let AmlValue::Package(s5) = aml.namespace.get_by_path(&name)? {
        if let AmlValue::Integer(value) = s5[BlockS5::SlpTypA as usize] {
            SLP_TYP_A.store(value as u16, Ordering::Relaxed);
        }
        if let AmlValue::Integer(value) = s5[BlockS5::SlpTypB as usize] {
            SLP_TYP_B.store(value as u16, Ordering::Relaxed);
        }
    }

    Ok(())
}

/// Returns the value of SLP_TYP_A register.
pub fn slp_typ_a() -> u16 { SLP_TYP_A.load(Ordering::Relaxed) }

/// Returns the value of SLP_TYP_B register.
pub fn slp_typ_b() -> u16 { SLP_TYP_B.load(Ordering::Relaxed) }

//////////////////////////
/// Custom AML Handler
//////////////////////////
#[derive(Clone)]
struct CustomAMLHandler;

impl Handler for CustomAMLHandler {
    fn read_u8(&self, address: usize) -> u8 { super::read_addr::<u8>(address) }

    fn read_u16(&self, address: usize) -> u16 { super::read_addr::<u16>(address) }

    fn read_u32(&self, address: usize) -> u32 { super::read_addr::<u32>(address) }

    fn read_u64(&self, address: usize) -> u64 { super::read_addr::<u64>(address) }

    fn write_u8(&mut self, _address: usize, _value: u8) { unimplemented!() }

    fn write_u16(&mut self, _address: usize, _value: u16) { unimplemented!() }

    fn write_u32(&mut self, _address: usize, _value: u32) { unimplemented!() }

    fn write_u64(&mut self, _address: usize, _value: u64) { unimplemented!() }

    fn read_io_u8(&self, _port: u16) -> u8 { unimplemented!() }

    fn read_io_u16(&self, _port: u16) -> u16 { unimplemented!() }

    fn read_io_u32(&self, _port: u16) -> u32 { unimplemented!() }

    fn write_io_u8(&self, _port: u16, _value: u8) { unimplemented!() }

    fn write_io_u16(&self, _port: u16, _value: u16) { unimplemented!() }

    fn write_io_u32(&self, _port: u16, _value: u32) { unimplemented!() }

    fn read_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u8 { unimplemented!() }

    fn read_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u16 { unimplemented!() }

    fn read_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u32 { unimplemented!() }

    fn write_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u8) { unimplemented!() }

    fn write_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u16) { unimplemented!() }

    fn write_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u32) { unimplemented!() }
}
