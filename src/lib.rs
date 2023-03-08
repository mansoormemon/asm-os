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

#[cfg(test)]
use crate::aux::testing::serene_test_panic_handler;
use crate::kernel::{acpi, allocator, gdt, interrupts, memory, vga_buffer};

pub mod aux;
pub mod kernel;

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);

    test_main();

    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serene_test_panic_handler(info);
}

pub fn init(boot_info: &'static BootInfo) {
    vga_buffer::clear();

    print!("Initialize GDT ... ");
    gdt::init();
    println!("[ ok ]");

    print!("Initialize interrupts ... ");
    interrupts::init();
    println!("[ ok ]");

    print!("Enabling interrupts ...");
    interrupts::enable();
    println!("[ ok ]");

    print!("Initialize memory ... ");
    memory::init(boot_info);
    println!("[ ok ]");

    print!("Initialize allocator ... ");
    allocator::init(boot_info);
    println!("[ ok ]");

    print!("Initialize ACPI ... ");
    acpi::init();
    println!("[ ok ]");
}

pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}
