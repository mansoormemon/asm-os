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
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(asm_os::aux::testing::serene_test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec::Vec;
use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};

use asm_os::{api, print, println};
use asm_os::api::kbd::Layout;
use asm_os::api::vga;
use asm_os::arch::x86::kernel::task::{Executor, Task};
#[cfg(test)]
use asm_os::aux::testing::serene_test_panic_handler;
use asm_os::dev::console;
#[cfg(not(test))]
use asm_os::hlt_loop;
use asm_os::init;

entry_point!(kernel_main);

pub async fn main() {
    loop {
        print!("\x1B[32m>\x1B[33m>\x1B[31m>\x1b[0m ");
        let response = console::read_line();
        let response = response.trim();
        if response == "clear" {
            vga::clear();
        } else if response.starts_with("kbd") {
            let tokens: Vec<&str> = response.split(" ").collect();
            if tokens[1] == "set" {
                if tokens[2] == "layout" {
                    let lyt = tokens[3];

                    match lyt {
                        "azerty" => api::kbd::set_layout(Layout::AZERTY),
                        "dvorak" => api::kbd::set_layout(Layout::Dvorak),
                        "qwerty" => api::kbd::set_layout(Layout::QWERTY),
                        _ => println!("\x1B[31mError:\x1B[0m Unknown layout"),
                    }
                } else {
                    println!("\x1B[31mError:\x1B[0m Unknown property")
                }
            } else if tokens[1] == "get" {
                if tokens[2] == "layout" {
                    println!("{}", api::kbd::get_layout().as_str());
                } else {
                    println!("\x1B[31mError:\x1B[0m Unknown property")
                }
            } else if tokens[1] == "reset" {
                if tokens[2] == "layout" {
                    api::kbd::reset_layout();
                } else {
                    println!("\x1B[31mError:\x1B[0m Unknown property")
                }
            } else {
                println!("\x1B[31mError:\x1B[0m Unknown operation")
            }
            println!("{}", tokens[2]);
        } else if response == "shutdown" {
            api::system::power_off();
        }
    }
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    vga::set_palette(vga::palette::MATERIAL_DARKER);
    init(boot_info, false);

    println!();
    println!("{}", format_args!("{: ^99}", "\x1B[34mWelcome to \x1B[35masmOS\x1B[34m!\x1B[0m"));
    println!();

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    // executor.spawn(Task::new(main()));
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
fn panic(info: &PanicInfo) -> ! { serene_test_panic_handler(info); }
