#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![feature(poll_ready)]
#![feature(exclusive_range_pattern)]
#![test_runner(crate::util::serene_test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(test)]
use core::panic::PanicInfo;

use bootloader::BootInfo;
#[cfg(test)]
use bootloader::entry_point;
use x86_64::{instructions, VirtAddr};

use crate::nub::{allocator, gdt, interrupts, memory, vga_buffer};
#[cfg(test)]
use crate::util::serene_test_panic_handler;
use crate::util::Unit;

pub mod emul;
pub mod nub;
pub mod util;

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
    println!("VGA Buffer initialized!");
    print!("Initialize GDT ... ");
    gdt::init();
    println!("[ ok ]");
    print!("Initialize IDT ... ");
    interrupts::init_idt();
    println!("[ ok ]");
    print!("Initialize PICS ... ");
    unsafe { interrupts::PICS.lock().initialize() };
    println!("[ ok ]");
    print!("Enable interrupts ... ");
    instructions::interrupts::enable();
    println!("[ ok ]");

    print!("Fetching PMO ... ");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    println!("{:?}", phys_mem_offset);
    print!("Map physical memory at PMO ... ");
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    println!("[ ok ]");
    print!("Initialize frame allocator ... ");
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    println!("[ ok ]");

    print!("Initialize heap ... ");
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("failed to initialize heap");
    println!("[ ok ]");
    println!("Heap={{ start={:#01x}, end={:#01x}, size={} {:?} }}", allocator::HEAP_START, allocator::HEAP_END, allocator::HEAP_SIZE / (Unit::KiB as usize), Unit::KiB);
}

pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}
