#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(asm_os::aux::testing::panicky_test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::BootInfo;
use volatile::Volatile;

use asm_os::{hlt_loop, init};
use asm_os::aux::testing::panicky_test_panic_handler;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    init(boot_info);

    test_main();

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panicky_test_panic_handler(info);
}

#[test_case]
fn test_unconditional_recursion() {
    #[allow(unconditional_recursion)]
    fn recurse() {
        recurse();
        Volatile::new(0).read();
    }

    recurse();
}
