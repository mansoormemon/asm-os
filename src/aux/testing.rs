use core::any::type_name;
use core::panic::PanicInfo;

use crate::{hlt_loop, serial_print, serial_println};
use crate::aux::emulator::{exit_qemu, QemuExitCode};

/// Serene Test
pub trait SereneTest {
    fn run(&self);
}

impl<T> SereneTest for T
    where T: Fn() {
    fn run(&self) {
        serial_print!("{} ... ", type_name::<T>());
        self();
        serial_println!("[ success ]");
    }
}

/// A runner for tests that are expected to complete calmly.
pub fn serene_test_runner(tests: &[&dyn SereneTest]) {
    serial_println!("Total tests: {}", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// A panic handler for serene tests.
pub fn serene_test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[ failure ]");
    serial_println!("{}", info);
    exit_qemu(QemuExitCode::Failure);
    hlt_loop();
}

/// Panicky Test
pub trait PanickyTest {
    fn run(&self);
}

impl<T> PanickyTest for T
    where T: Fn() {
    fn run(&self) {
        serial_print!("{} ... ", type_name::<T>());
        self();
        serial_println!("[ failure ]");
    }
}

/// A runner for tests that are expected to panic.
pub fn panicky_test_runner(tests: &[&dyn PanickyTest]) {
    serial_println!("Total tests: {}", tests.len());
    if tests.len() > 1 {
        serial_println!("WARNING: More than 1 tests found; only the first one will be run.");
    }
    if let Some(test) = tests.first() {
        test.run();
        exit_qemu(QemuExitCode::Failure);
    }
}

/// A panic handler for panicky tests.
pub fn panicky_test_panic_handler(_info: &PanicInfo) -> ! {
    serial_println!("[ success ]");
    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}
