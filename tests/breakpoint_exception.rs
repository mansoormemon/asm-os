#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(asm_os::aux::testing::serene_test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::BootInfo;
use x86_64::instructions;

use asm_os::{hlt_loop, init};
use asm_os::aux::testing::serene_test_panic_handler;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    init(boot_info);

    test_main();

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serene_test_panic_handler(info);
}

#[test_case]
fn test_x86_64_int3() {
    instructions::interrupts::int3();
}
