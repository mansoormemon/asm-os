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
use crate::kernel::{acpi, allocator, gdt, interrupts, keyboard, memory, pit, vga};

pub mod api;
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
    vga::init();

    print!("Initialize GDT ... ");
    gdt::init();
    println!("[ ok ]");

    print!("Initialize interrupts ... ");
    interrupts::init();
    println!("[ ok ]");

    print!("Enable interrupts ... ");
    interrupts::enable();
    println!("[ ok ]");

    print!("Initialize PIT ... ");
    pit::init();
    println!("[ ok ]");

    print!("Initialize logger ... ");
    logger::init(LogLevel::INFO);
    println!("[ ok ]");

    println!();

    log!( LogLevel::INFO , "Initialize memory ... ");
    memory::init(boot_info);
    println!("[ ok ]");

    log!(LogLevel::INFO , "Initialize allocator ... ");
    allocator::init(boot_info);
    println!("[ ok ]");

    log!( LogLevel::INFO, "Initialize ACPI ... ");
    if let Err(err_code) = acpi::init() {
        println!("[ failure ], error={:?}", err_code);
    } else {
        println!("[ ok ]");
    }

    log!( LogLevel::INFO, "Initialize keyboard ... ");
    keyboard::init();
    println!(" [ ok ] ");

    println!();
}

/// Halts execution of CPU until next interrupt.
pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}
