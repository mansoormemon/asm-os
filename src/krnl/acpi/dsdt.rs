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

///////////////////
// Cached Values
///////////////////

use alloc::boxed::Box;
use core::slice;
use core::sync::atomic::{AtomicU16, Ordering};

use acpi::AmlTable;
use aml::{AmlContext, AmlName, AmlValue, DebugVerbosity};
use aml::Handler;
use x86_64::PhysAddr;

use crate::krnl::memory;
use crate::warning;

/// Parsed value of SLP_TYPA from the AML tables.
pub(crate) static SLP_TYPA: AtomicU16 = AtomicU16::new(u16::MAX);
/// Parsed value of SLP_TYPB from the AML tables.
pub(super) static SLP_TYPB: AtomicU16 = AtomicU16::new(u16::MAX);

/// Value of SLP_EN.
pub(crate) const SLP_EN: u16 = 0x2000;

/// Block code for S5.
const BLOCK_CODE_S5: &'static str = "\\_S5";

////////////////
/// Block S5
////////////////
#[repr(u8)]
enum BlockS5 {
    SLPTYPA,
    SLPTYPB,
}

pub(super) fn parse(sdt: AmlTable) {
    let mut aml = AmlContext::new(Box::new(CustomAMLHandler), DebugVerbosity::None);

    let address = memory::phys_to_virt_addr(PhysAddr::new(sdt.address as u64));
    let stream = unsafe { slice::from_raw_parts(address.as_ptr(), sdt.length as usize) };

    if let Ok(_) = aml.parse_table(stream) {
        let name = AmlName::from_str(BLOCK_CODE_S5).unwrap();
        if let Ok(AmlValue::Package(s5)) = aml.namespace.get_by_path(&name) {
            if let AmlValue::Integer(value) = s5[BlockS5::SLPTYPA as usize] {
                SLP_TYPA.store(value as u16, Ordering::Relaxed);
            }
            if let AmlValue::Integer(value) = s5[BlockS5::SLPTYPB as usize] {
                SLP_TYPB.store(value as u16, Ordering::Relaxed);
            }
        } else {
            warning!("ACPI: D={:?}", super::CustomACPIError::S5ParseFailure);
        }
    } else {
        warning!("ACPI: D={:?}", super::CustomACPIError::AMLParseFailure);
    }
}

//////////////////////////
/// Custom AML Handler
//////////////////////////
#[derive(Clone)]
struct CustomAMLHandler;

impl Handler for CustomAMLHandler {
    fn read_u8(&self, address: usize) -> u8 {
        super::read_addr::<u8>(address)
    }

    fn read_u16(&self, address: usize) -> u16 {
        super::read_addr::<u16>(address)
    }

    fn read_u32(&self, address: usize) -> u32 {
        super::read_addr::<u32>(address)
    }

    fn read_u64(&self, address: usize) -> u64 {
        super::read_addr::<u64>(address)
    }

    fn write_u8(&mut self, _address: usize, _value: u8) {
        unimplemented!()
    }

    fn write_u16(&mut self, _address: usize, _value: u16) {
        unimplemented!()
    }

    fn write_u32(&mut self, _address: usize, _value: u32) {
        unimplemented!()
    }

    fn write_u64(&mut self, _address: usize, _value: u64) {
        unimplemented!()
    }

    fn read_io_u8(&self, _port: u16) -> u8 {
        unimplemented!()
    }

    fn read_io_u16(&self, _port: u16) -> u16 {
        unimplemented!()
    }

    fn read_io_u32(&self, _port: u16) -> u32 {
        unimplemented!()
    }

    fn write_io_u8(&self, _port: u16, _value: u8) {
        unimplemented!()
    }

    fn write_io_u16(&self, _port: u16, _value: u16) {
        unimplemented!()
    }

    fn write_io_u32(&self, _port: u16, _value: u32) {
        unimplemented!()
    }

    fn read_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u8 {
        unimplemented!()
    }

    fn read_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u16 {
        unimplemented!()
    }

    fn read_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u32 {
        unimplemented!()
    }

    fn write_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u8) {
        unimplemented!()
    }

    fn write_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u16) {
        unimplemented!()
    }

    fn write_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u32) {
        unimplemented!()
    }
}
