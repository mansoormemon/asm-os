use x86_64::instructions::port::Port;

/// QEMU Exit Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QEMUExitCode {
    Success = 0x10,
    Failure = 0x11,
}

/// Exits QEMU with the given exit code.
pub fn exit_qemu(exit_code: QEMUExitCode) {
    const PORT: u16 = 0xF4;

    let mut port = Port::new(PORT);
    unsafe {
        port.write(exit_code as u32);
    }
}
