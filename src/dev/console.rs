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

//

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
    const ETX: char = ascii::ETX as char;
    const EOT: char = ascii::EOT as char;
    const BS: char = ascii::BS as char;
    const ESC: char = ascii::ESC as char;

    let mut stdin = BUFFER.lock();

    if key == BS && !is_raw_enabled() {
        if let Some(c) = stdin.pop() {
            if is_echo_enabled() {
                let n = match c {
                    ETX | EOT | ESC => 2,
                    _ => if (c as u32) < 0xFF { 1 } else { c.len_utf8() },
                };
                print!("{}", BS.to_string().repeat(n));
            }
        }
    } else {
        let key = if (key as u32) < 0xFF { (key as u8) as char } else { key };
        stdin.push(key);
        if is_echo_enabled() {
            match key {
                ETX => print!("^C"),
                EOT => print!("^D"),
                ESC => print!("^["),
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
        let res = instructions::interrupts::without_interrupts(|| {
            let mut stdin = BUFFER.lock();
            if !stdin.is_empty() {
                Some(stdin.remove(0))
            } else {
                None
            }
        });
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
        let res = instructions::interrupts::without_interrupts(|| {
            let mut stdin = BUFFER.lock();
            match stdin.chars().next_back() {
                Some('\n') => {
                    let line = stdin.clone();
                    stdin.clear();
                    Some(line)
                }
                _ => {
                    None
                }
            }
        });
        if let Some(line) = res {
            return line;
        }
    }
}
