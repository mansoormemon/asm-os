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

use core::any;
use core::panic::PanicInfo;

use crate::{serial_print, serial_println};
use crate::aux::emulator::qemu;
use crate::hlt_loop;

///////////////////
/// Serene Test
///////////////////
pub trait SereneTest {
    /// The run function.
    fn run(&self);
}

impl<T> SereneTest for T
    where T: Fn() {
    fn run(&self) {
        serial_print!("{} ... ", any::type_name::<T>());
        self();
        serial_println!("\x1B[32m[ success ]\x1B[0m");
    }
}

/// A runner for tests that are expected to complete calmly.
pub fn serene_test_runner(tests: &[&dyn SereneTest]) {
    serial_println!("Total tests: {}", tests.len());
    for test in tests {
        test.run();
    }
    qemu::exit(qemu::ExitCode::Success);
}

/// A panic handler for serene tests.
pub fn serene_test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("\x1B[31m[ failure ]\x1B[0m");
    serial_println!("{}", info);
    qemu::exit(qemu::ExitCode::Failure);
    hlt_loop();
}

////////////////////
/// Panicky Test
////////////////////
pub trait PanickyTest {
    /// The run function.
    fn run(&self);
}

impl<T> PanickyTest for T
    where T: Fn() {
    fn run(&self) {
        serial_print!("{} ... ", any::type_name::<T>());
        self();
        serial_println!("\x1B[31m[ failure ]\x1B[0m");
    }
}

/// A runner for tests that are expected to panic.
pub fn panicky_test_runner(tests: &[&dyn PanickyTest]) {
    serial_println!("Total tests: {}", tests.len());
    if tests.len() > 1 {
        serial_println!("\x1B[93mWARNING: More than 1 tests found; only the first one will be run.\x1B[0m");
    }
    if let Some(test) = tests.first() {
        test.run();
        qemu::exit(qemu::ExitCode::Failure);
    }
}

/// A panic handler for panicky tests.
pub fn panicky_test_panic_handler(_info: &PanicInfo) -> ! {
    serial_println!("\x1B[32m[ success ]\x1B[0m");
    qemu::exit(qemu::ExitCode::Success);
    hlt_loop();
}
