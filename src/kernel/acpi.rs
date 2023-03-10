use alloc::boxed::Box;
use core::ptr::NonNull;
use core::slice;
use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU8, Ordering};

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use aml::{AmlContext, AmlName, AmlValue, DebugVerbosity, Handler};
use x86_64::instructions::port::Port;
use x86_64::PhysAddr;

use crate::kernel::memory;

/// Differentiated System Description Table.
pub const DSDT: &'static str = "DSDT";
/// Fixed ACPI Description Table.
pub const FADT: &'static str = "FACP";
/// Multiple APIC Description Table.
pub const MADT: &'static str = "APIC";
/// Root System Description Pointer.
pub const RSDP: &'static str = "RSD PTR";
/// Extended System Description Table.
pub const XSDT: &'static str = "XSDT";

/// Fixed ACPI Description Table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FADT {
    Reserved = 44,
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

/// Value of SCI interrupt in the FADT register.
static SCI_INTERRUPT: AtomicU16 = AtomicU16::new(u16::MAX);
/// Value of SMI command port in the FADT register.
static SMI_COMMAND_PORT: AtomicU32 = AtomicU32::new(u32::MAX);
/// Value of ACPI enable in the FADT register.
static ACPI_ENABLE: AtomicU8 = AtomicU8::new(u8::MAX);
/// Value of ACPI disable in the FADT register.
static ACPI_DISABLE: AtomicU8 = AtomicU8::new(u8::MAX);
/// Value of PM-1A control block in the FADT register.
static PM_1A_CONTROL_BLOCK: AtomicU32 = AtomicU32::new(u32::MAX);
/// Value of PM-1B control block in the FADT register.
static PM_1B_CONTROL_BLOCK: AtomicU32 = AtomicU32::new(u32::MAX);

/// Parsed value of SLP_TYPA from the AML tables.
static SLP_TYPA: AtomicU16 = AtomicU16::new(u16::MAX);
/// Parsed value of SLP_TYPB from the AML tables.
static SLP_TYPB: AtomicU16 = AtomicU16::new(u16::MAX);

/// Value of SLP_EN.
const SLP_EN: u16 = 0x2000;

/// Block code for S5.
const BLOCK_CODE_S5: &'static str = "\\_S5";

/// Block S5.
#[repr(u8)]
enum BlockS5 {
    SLPTYPA,
    SLPTYPB,
}

/// Initializes the ACPI and stores required parameters.
pub fn init() -> Result<(), CustomAcpiError> {
    if let Ok(acpi) = unsafe { AcpiTables::search_for_rsdp_bios(CustomAcpiHandler) } {
        let mut fadt_found = false;
        for (sign, sdt) in acpi.sdts {
            match sign.as_str() {
                FADT => {
                    fadt_found = true;
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

        if !fadt_found {
            return Err(CustomAcpiError::FADTNotFound);
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
                    } else {
                        return Err(CustomAcpiError::S5ParseFailure);
                    }
                } else {
                    return Err(CustomAcpiError::AMLParseFailure);
                }
            }
            None => {
                return Err(CustomAcpiError::InvalidDSDT);
            }
        }
    } else {
        return Err(CustomAcpiError::ACPINotFound);
    }

    Ok(())
}

/// Converts the given physical address to virtual address and returns it.
fn read_addr<T>(phys_addr: usize) -> T where T: Copy {
    let virt_addr = memory::phys_to_virt_addr(PhysAddr::new(phys_addr as u64));
    unsafe { *virt_addr.as_ptr::<T>() }
}

/// Reads the value of the given register and returns it.
fn read_fadt<T>(fadt_phys_addr: usize, register: FADT) -> T where T: Copy {
    read_addr(fadt_phys_addr + register as usize)
}

/// Custom ACPI Handler.
#[derive(Clone)]
struct CustomAcpiHandler;

impl AcpiHandler for CustomAcpiHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let virt_addr = memory::phys_to_virt_addr(PhysAddr::new(physical_address as u64));
        PhysicalMapping::new(physical_address, NonNull::new(virt_addr.as_mut_ptr()).unwrap(), size, size, Self)
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

/// Custom AML Handler.
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

/// Custom ACPI Error.
#[derive(Debug, Clone, Copy)]
pub enum CustomAcpiError {
    ACPINotFound,
    AMLParseFailure,
    InvalidDSDT,
    FADTNotFound,
    S5ParseFailure,
}

/// Powers off the machine.
pub fn power_off() {
    let pm_1a_cnt_blk = PM_1A_CONTROL_BLOCK.load(Ordering::Relaxed);
    let slp_typa = SLP_TYPA.load(Ordering::Relaxed);

    let mut port_pm_1a = Port::new(pm_1a_cnt_blk as u16);
    unsafe {
        port_pm_1a.write(slp_typa | SLP_EN);
    }
}
