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

use alloc::string::{String, ToString};
use core::sync::atomic::{AtomicBool, Ordering};

use spin::Mutex;
use x86_64::instructions;

use crate::api::system;
use crate::cenc::ascii;
use crate::print;

// todo: complete later; we need filesystem first.

static BUFFER: Mutex<String> = Mutex::new(String::new());

/// Echo enabled.
static ECHO_ENABLED: AtomicBool = AtomicBool::new(true);

static RAW_ENABLED: AtomicBool = AtomicBool::new(false);

pub(crate) fn is_echo_enabled() -> bool { ECHO_ENABLED.load(Ordering::SeqCst) }

pub(crate) fn enable_echo() { ECHO_ENABLED.store(true, Ordering::SeqCst); }

pub(crate) fn disable_echo() { ECHO_ENABLED.store(false, Ordering::SeqCst); }

pub(crate) fn is_raw_enabled() -> bool { RAW_ENABLED.load(Ordering::SeqCst) }

pub(crate) fn enable_raw() { RAW_ENABLED.store(true, Ordering::SeqCst); }

pub(crate) fn disable_raw() { RAW_ENABLED.store(false, Ordering::SeqCst); }

pub fn key_handle(key: char) {
    let mut stdin = BUFFER.lock();

    if key == ascii::ch::BS && !is_raw_enabled() {
        if let Some(c) = stdin.pop() {
            if is_echo_enabled() {
                let n = match c {
                    ascii::ch::ETX | ascii::ch::EOT | ascii::ch::ESC => 2,
                    _ => if (c as u32) < 0xFF { 1 } else { c.len_utf8() },
                };
                print!("{}", ascii::ch::BS.to_string().repeat(n));
            }
        }
    } else {
        let key = if (key as u32) < 0xFF { (key as u8) as char } else { key };
        stdin.push(key);
        if is_echo_enabled() {
            match key {
                ascii::ch::ETX => print!("^C"),
                ascii::ch::EOT => print!("^D"),
                ascii::ch::ESC => print!("^["),
                _ => print!("{}", key),
            };
        }
    }
}

pub fn read_char() -> char {
    disable_echo();
    enable_raw();
    loop {
        system::halt();
        let res = instructions::interrupts::without_interrupts(
            || {
                let mut buffer = BUFFER.lock();
                if !buffer.is_empty() {
                    Some(buffer.remove(0))
                } else {
                    None
                }
            }
        );
        if let Some(c) = res {
            enable_echo();
            disable_raw();
            return c;
        }
    }
}

pub fn read_line() -> String {
    loop {
        system::halt();
        let res = instructions::interrupts::without_interrupts(
            || {
                let mut stdin = BUFFER.lock();
                match stdin.chars().next_back() {
                    Some(ascii::ch::CR) | Some(ascii::ch::LF) | Some(ascii::ch::FF) => {
                        let line = stdin.clone();
                        stdin.clear();
                        Some(line)
                    }
                    _ => {
                        None
                    }
                }
            }
        );
        if let Some(line) = res {
            return line;
        }
    }
}
