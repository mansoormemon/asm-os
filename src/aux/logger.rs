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

use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions;

use crate::{print, println};
use crate::api::system;
use crate::api::vga;

///////////////////////
// Local Interfaces
///////////////////////

lazy_static! {
    /// A global interface for our logger.
    static ref LOGGER : Mutex<Logger> = Mutex::new(Logger::new());
}

/////////////////
/// Log Level
/////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Quiet = 0x0,
    Failure = 0x1,
    Warning = 0x2,
    Success = 0x3,
    Apprise = 0x4,
}

//////////////
/// Logger
//////////////
struct Logger {
    log_level: LogLevel,
}

impl Logger {
    /// Creates a new object.
    fn new() -> Self {
        Logger {
            log_level: LogLevel::Apprise,
        }
    }

    /// Returns the log level.
    fn get_log_level(&self) -> LogLevel { return self.log_level; }

    /// Sets the log level.
    fn set_log_level(&mut self, log_level: LogLevel) { self.log_level = log_level; }
}

/// Returns the log level.
pub fn get_log_level() -> LogLevel {
    instructions::interrupts::without_interrupts(
        || { LOGGER.lock().get_log_level() }
    )
}

/// Sets the log level.
pub fn set_log_level(log_level: LogLevel) {
    instructions::interrupts::without_interrupts(
        || { LOGGER.lock().set_log_level(log_level); }
    );
}

///////////////
// Utilities
///////////////

/// Initializes the logger.
pub(crate) fn init(log_level: LogLevel) {
    set_log_level(log_level);
}

#[doc(hidden)]
pub fn _log(log_level: LogLevel, fmt: fmt::Arguments) {
    if get_log_level() < log_level { return; }

    const PRECISION: usize = 4;
    const STATUS_MARK_LENGTH: usize = 10;
    const UPTIME_LENGTH: usize = 13;

    if system::is_timer_initialized() {
        print!("\x1B[93m[{:01$.02$}] ", system::uptime(), UPTIME_LENGTH, PRECISION);
    } else {
        print!("\x1B[93m[--------.----] ");
    }

    print!("\x1B[0m{} ", fmt);

    let (_, col) = vga::get_cursor_pos();
    for _ in col..(vga::cols() - STATUS_MARK_LENGTH) {
        print!(".");
    }

    match log_level {
        LogLevel::Failure => {
            println!(" \x1B[31m[failure]\x1B[0m");
        }
        LogLevel::Warning => {
            println!(" \x1B[33m[warning]\x1B[0m");
        }
        LogLevel::Success => {
            println!(" \x1B[32m[success]\x1B[0m");
        }
        LogLevel::Apprise => {
            println!(" \x1B[34m[apprise]\x1B[0m");
        }
        _ => {}
    }
}

#[doc(hidden)]
pub fn _failure(fmt: fmt::Arguments) { _log(LogLevel::Failure, fmt); }

#[doc(hidden)]
pub fn _warning(fmt: fmt::Arguments) { _log(LogLevel::Warning, fmt); }

#[doc(hidden)]
pub fn _success(fmt: fmt::Arguments) { _log(LogLevel::Success, fmt); }

#[doc(hidden)]
pub fn _apprise(fmt: fmt::Arguments) { _log(LogLevel::Apprise, fmt); }

////////////
// Macros
////////////

#[macro_export]
macro_rules! log {
    ($log_level:expr, $($arg:tt)*) => ($crate::aux::logger::_log($log_level, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! failure {
    ($($arg:tt)*) => ($crate::aux::logger::_failure(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => ($crate::aux::logger::_warning(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => ($crate::aux::logger::_success(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! apprise {
    ($($arg:tt)*) => ($crate::aux::logger::_apprise(format_args!($($arg)*)));
}
