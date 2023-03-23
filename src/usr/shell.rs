// MIT License
//
// Copyright (c) 2023 Mansoor Ahmed Memon.
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

use alloc::vec::Vec;

use crate::{print, println};
use crate::api::io;
use crate::usr::{chrono, clear, kbd, list, reboot, shutdown, vga};

pub async fn main() {
    loop {
        print!("\x1B[32m@\x1b[0m ");
        let response = io::stdin().read_line();
        let response = response.trim();
        if response == "" { continue; }
        let tokens: Vec<&str> = response.split(' ').collect();
        exec(tokens.as_slice());
    }
}

fn exec(tokens: &[&str]) {
    let commands: [(&str, fn(&[&str])); 7] = [
        ("clear", clear::main),
        ("shutdown", shutdown::main),
        ("reboot", reboot::main),
        ("chrono", chrono::main),
        ("list", list::main),
        ("vga", vga::main),
        ("kbd", kbd::main),
    ];
    let mut cmd_found = false;
    match tokens {
        [exe, args @ ..] => {
            for (cmd, exe_func) in commands {
                if *exe == cmd {
                    exe_func(args);
                    cmd_found = true;
                    break;
                }
            }
        }
        [] => {}
    };
    if !cmd_found {
        println!("\x1B[31mError:\x1B[0m command `{}` not found", tokens[0]);
    }
}
