use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions;

lazy_static! {
    /// Global interface for serial outputting to host system.
    static ref SERIAL_3F8: Mutex<SerialPort> = {
        const PORT_ADDR: u16 = 0x3F8;

        let mut serial_port = unsafe { SerialPort::new(PORT_ADDR) };
        serial_port.init();

        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;

    instructions::interrupts::without_interrupts(
        || { SERIAL_3F8.lock().write_fmt(args).expect("Could not print to serial output."); }
    );
}

// Macros

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::kernel::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}
