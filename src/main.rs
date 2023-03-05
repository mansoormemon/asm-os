#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(asm_os::util::serene_test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};

use asm_os::{init, println};
#[cfg(not(test))]
use asm_os::hlt_loop;
use asm_os::nub::task::{self, Task};
use asm_os::nub::task::executor::Executor;
#[cfg(test)]
use asm_os::util::serene_test_panic_handler;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);

    println!("START");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(task::keyboard::echo_key_presses()));
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
