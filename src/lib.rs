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

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![feature(poll_ready)]
#![feature(exclusive_range_pattern)]
#![test_runner(crate::aux::testing::serene_test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(test)]
use core::panic::PanicInfo;

use bootloader::BootInfo;
#[cfg(test)]
use bootloader::entry_point;
use x86_64::instructions;

use crate::aux::logger;
use crate::aux::logger::LogLevel;
#[cfg(test)]
use crate::aux::testing::serene_test_panic_handler;

pub mod api;
pub mod aux;
pub mod cenc;
pub mod dev;
pub mod drv;
pub mod krnl;
pub mod usr;

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info, false);
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! { serene_test_panic_handler(info); }

/// Initializes all sub-modules.
pub fn init(boot_info: &'static BootInfo, silent_boot: bool) {
    drv::output::vga::init();

    logger::init(if silent_boot { LogLevel::Quiet } else { LogLevel::Apprise });

    krnl::gdt::init();
    krnl::idt::init();
    krnl::pics::init();
    krnl::pics::enable();

    krnl::pit::init();

    krnl::memory::init(boot_info);
    krnl::allocator::init(boot_info);
    krnl::acpi::init();

    drv::input::kbd::init(api::kbd::Layout::QWERTY);
}

/// Halts execution of CPU until next interrupt.
pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}
