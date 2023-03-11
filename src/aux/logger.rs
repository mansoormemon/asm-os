use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions;

use crate::api::vga::{self, color::Color};
use crate::kernel::pit;
use crate::print;

/// Log Level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    QUIET,
    INFO,
    WARNING,
    ERROR,
}

/// Logger.
struct Logger {
    log_level: LogLevel,
}

impl Logger {
    /// Creates a new object.
    fn new() -> Self {
        Logger {
            log_level: LogLevel::INFO,
        }
    }

    /// Returns the log level.
    fn get_log_level(&self) -> LogLevel {
        return self.log_level;
    }

    /// Sets the log level.
    fn set_log_level(&mut self, log_level: LogLevel) {
        self.log_level = log_level;
    }
}

lazy_static! {
    /// A global interface for our logger.
    static ref LOGGER : Mutex<Logger> = Mutex::new(Logger::new());
}

/// A safe interface for returning the log level.
pub fn get_log_level() -> LogLevel {
    instructions::interrupts::without_interrupts(
        || { LOGGER.lock().get_log_level() }
    )
}

/// A safe interface for setting the log level.
pub fn set_log_level(log_level: LogLevel) {
    instructions::interrupts::without_interrupts(
        || { LOGGER.lock().set_log_level(log_level); }
    );
}

/// Initializes the logger.
pub fn init(log_level: LogLevel) {
    set_log_level(log_level);
}

#[doc(hidden)]
pub fn _log(log_level: LogLevel, fmt: fmt::Arguments) {
    if log_level >= get_log_level() {
        vga::set_foreground(Color::Green);
        print!("[ {:01$.02$} ] ", pit::uptime(), 13, 4);
        vga::reset_foreground();
        print!("{}", fmt);
    }
}

// Macros

#[macro_export]
macro_rules! log {
    ($log_level:expr, $($arg:tt)*) => ($crate::aux::logger::_log($log_level, format_args!($($arg)*)));
}
