use alloc::boxed::Box;
use core::ptr::NonNull;
use core::slice;
use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU8, Ordering};

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use aml::{AmlContext, AmlName, AmlValue, DebugVerbosity, Handler};
use x86_64::instructions::port::Port;
use x86_64::PhysAddr;

use crate::kernel::memory;

pub const DSDT: &'static str = "DSDT";
pub const FADT: &'static str = "FACP";
pub const GTDT: &'static str = "GTDT";
pub const MADT: &'static str = "APIC";
pub const MCFG: &'static str = "MCFG";
pub const RSDP: &'static str = "RSD PTR";
pub const SPCR: &'static str = "SPCR";
pub const XSDT: &'static str = "XSDT";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FADT {
    Reserved = 44,
    PreferredPowerManagementProfile = 45,
    SCIInterrupt = 46,
    SMICommandPort = 48,
    ACPIEnable = 52,
    ACPIDisable = 53,
    S4BIOSREQ = 54,
    PSTATEControl = 55,
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

static SCI_INTERRUPT: AtomicU16 = AtomicU16::new(u16::MAX);
static SMI_COMMAND_PORT: AtomicU32 = AtomicU32::new(u32::MAX);
static ACPI_ENABLE: AtomicU8 = AtomicU8::new(u8::MAX);
static ACPI_DISABLE: AtomicU8 = AtomicU8::new(u8::MAX);

static PM_1A_CONTROL_BLOCK: AtomicU32 = AtomicU32::new(u32::MAX);
static PM_1B_CONTROL_BLOCK: AtomicU32 = AtomicU32::new(u32::MAX);

static SLP_TYPA: AtomicU16 = AtomicU16::new(u16::MAX);
static SLP_TYPB: AtomicU16 = AtomicU16::new(u16::MAX);

static SLP_EN: AtomicU16 = AtomicU16::new(u16::MAX);

const BLOCK_CODE_S5: &'static str = "\\_S5";

const PM_1X_CONTROL_BLOCK_BIT: u8 = 13;

#[repr(u8)]
enum BlockS5 {
    SLPTYPA,
    SLPTYPB,
}

pub fn init() {
    if let Ok(acpi) = unsafe { AcpiTables::search_for_rsdp_bios(CustomAcpiHandler) } {
        for (sign, sdt) in acpi.sdts {
            match sign.as_str() {
                FADT => {
                    SCI_INTERRUPT.store(
                        read_fadt(sdt.physical_address, FADT::SCIInterrupt),
                        Ordering::Relaxed,
                    );
                    SMI_COMMAND_PORT.store(
                        read_fadt(sdt.physical_address, FADT::SMICommandPort),
                        Ordering::Relaxed,
                    );
                    ACPI_ENABLE.store(
                        read_fadt(sdt.physical_address, FADT::ACPIEnable),
                        Ordering::Relaxed,
                    );
                    ACPI_DISABLE.store(
                        read_fadt(sdt.physical_address, FADT::ACPIDisable),
                        Ordering::Relaxed,
                    );
                    PM_1A_CONTROL_BLOCK.store(
                        read_fadt(sdt.physical_address, FADT::PM1AControlBlock),
                        Ordering::Relaxed,
                    );
                    PM_1B_CONTROL_BLOCK.store(
                        read_fadt(sdt.physical_address, FADT::PM1BControlBlock),
                        Ordering::Relaxed,
                    );
                }
                _ => {}
            }
        }

        match &acpi.dsdt {
            Some(dsdt) => {
                let mut aml = AmlContext::new(Box::new(CustomAmlHandler), DebugVerbosity::None);

                let address = memory::phys_to_virt_addr(PhysAddr::new(dsdt.address as u64));
                let stream = unsafe { slice::from_raw_parts(address.as_ptr(), dsdt.length as usize) };

                if aml.parse_table(stream).is_ok() {
                    let name = AmlName::from_str(BLOCK_CODE_S5).unwrap();
                    if let Ok(AmlValue::Package(s5)) = aml.namespace.get_by_path(&name) {
                        if let AmlValue::Integer(value) = s5[BlockS5::SLPTYPA as usize] {
                            SLP_TYPA.store(value as u16, Ordering::Relaxed);
                        }
                        if let AmlValue::Integer(value) = s5[BlockS5::SLPTYPB as usize] {
                            SLP_TYPB.store(value as u16, Ordering::Relaxed);
                        }
                    }
                    SLP_EN.store(1 << PM_1X_CONTROL_BLOCK_BIT, Ordering::Relaxed);
                }
            }
            None => {}
        }
    }
}

fn read_addr<T>(phys_addr: usize) -> T where T: Copy {
    let virt_addr = memory::phys_to_virt_addr(PhysAddr::new(phys_addr as u64));
    unsafe { *virt_addr.as_ptr::<T>() }
}

fn read_fadt<T>(phys_addr: usize, register: FADT) -> T where T: Copy {
    read_addr(phys_addr + register as usize)
}

#[derive(Clone)]
struct CustomAcpiHandler;

impl AcpiHandler for CustomAcpiHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let virt_addr = memory::phys_to_virt_addr(PhysAddr::new(physical_address as u64));
        PhysicalMapping::new(physical_address, NonNull::new(virt_addr.as_mut_ptr()).unwrap(), size, size, Self)
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

#[derive(Clone)]
struct CustomAmlHandler;

impl Handler for CustomAmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        read_addr::<u8>(address)
    }

    fn read_u16(&self, address: usize) -> u16 {
        read_addr::<u16>(address)
    }

    fn read_u32(&self, address: usize) -> u32 {
        read_addr::<u32>(address)
    }

    fn read_u64(&self, address: usize) -> u64 {
        read_addr::<u64>(address)
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

pub fn power_off() {
    let pm_1a_cnt_blk = PM_1A_CONTROL_BLOCK.load(Ordering::Relaxed);
    let slp_typa = SLP_TYPA.load(Ordering::Relaxed);

    let pm_1b_cnt_blk = PM_1B_CONTROL_BLOCK.load(Ordering::Relaxed);
    let slp_typb = SLP_TYPB.load(Ordering::Relaxed);

    let slp_en = SLP_EN.load(Ordering::Relaxed);

    let mut port_pm_1a = Port::new(pm_1a_cnt_blk as u16);
    let mut port_pm_1b = Port::new(pm_1b_cnt_blk as u16);
    unsafe {
        port_pm_1a.write(slp_typa | slp_en);
        if pm_1b_cnt_blk != 0 {
            port_pm_1b.write(slp_typb | slp_en);
        }
    }
}
