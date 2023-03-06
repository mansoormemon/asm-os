#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(asm_os::aux::testing::serene_test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};

use asm_os::{init, println};
#[cfg(test)]
use asm_os::aux::testing::serene_test_panic_handler;
#[cfg(not(test))]
use asm_os::hlt_loop;
use asm_os::kernel::keyboard;
use asm_os::kernel::task::executor::Executor;
use asm_os::kernel::task::Task;

entry_point!(kernel_main);

pub async fn main() {}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);

    println!("Welcome to asmOS!");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::echo()));
    executor.spawn(Task::new(main()));
    executor.run();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serene_test_panic_handler(info);
}
