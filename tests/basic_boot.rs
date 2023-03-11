#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(asm_os::aux::testing::serene_test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use asm_os::{hlt_loop, println};
use asm_os::api::vga;
use asm_os::api::vga::color::Color;
use asm_os::aux::testing::serene_test_panic_handler;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serene_test_panic_handler(info);
}

#[test_case]
fn test_println() {
    println!("test_println output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    vga::clear();
    let s = "The quick brown fox jumps over the lazy dog.";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let (screen_char, _) = vga::query_data_at(0, i).unwrap();
        assert_eq!(screen_char as char, c);
    }
}

#[test_case]
fn test_coloring() {
    vga::set_color_code(Color::Yellow, Color::Blue);
    vga::clear();

    let (_, color_code) = vga::query_data_at(0, 0).unwrap();
    let expected_color_code = (Color::Blue as u8) << 4 | (Color::Yellow as u8);
    assert_eq!(color_code, expected_color_code);
}
