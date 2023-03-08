use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions;
use x86_64::instructions::port::Port;

// VGA BUFFER
//
// The VGA text buffer is a two-dimensional array with typically 25 rows and 80 columns, which is
// directly rendered to the screen. Each screen character is represented by two bytes aligned as a
// 16-bit word accessible by the CPU in a single operation. The lower, or character, byte is the
// actual code point for the current character set, and the higher, or attribute, byte is a bit
// field used to select various video attributes.
//
// Character Encoding: Code Page 437
//
// Each array entry describes a single screen character through the following format:
//
// Format: BLINK(15 : 1) + BG(14..12 : 3) + FG(11..8 : 4) + ASCII(7..0 : 8) [ MSB(15)..LSB(0) ]
//
// Wikipedia: https://en.wikipedia.org/wiki/VGA_text_mode

// White-Space Characters

pub const CHAR_SPACE: u8 = b' ';
pub const CHAR_NEWLINE: u8 = b'\n';
pub const CHAR_TAB: u8 = b'\t';
pub const CHAR_BACKSPACE: u8 = b'\x08';
pub const CHAR_FORM_FEED: u8 = b'\x0C';
pub const CHAR_CARRIAGE_RETURN: u8 = b'\r';

// Character Configurations

pub const TAB_WIDTH: u8 = 8;

// Ranges

pub const RANGE_PRINTABLE_ASCII_BEGIN: u8 = 0x20;
pub const RANGE_PRINTABLE_ASCII_END: u8 = 0x7E;

// Fallback Case

pub const FALLBACK_CHAR: u8 = 0xFE;

/// Color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

/// Color Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    const FG_COLOR_BIT_COUNT: u8 = 4;
    const FG_COLOR_BIT_MASK: u8 = 0xF;

    fn new(fg_color: Color, bg_color: Color) -> ColorCode {
        ColorCode((bg_color as u8) << Self::FG_COLOR_BIT_COUNT | (fg_color as u8))
    }

    fn fg(&self) -> u8 {
        self.0 & Self::FG_COLOR_BIT_MASK
    }

    fn bg(&self) -> u8 {
        self.0 >> Self::FG_COLOR_BIT_COUNT
    }

    fn as_u8(&self) -> u8 {
        self.0
    }
}

/// Screen Character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

// VGA Buffer Attributes

/// The VGA text buffer can be accessed via memory-mapped I/O at 0xB8000.
pub const ADDRESS: isize = 0xB8000;
/// The VGA text buffer is typically 80 columns wide.
pub const WIDTH: usize = 80;
/// The VGA text buffer is typically 25 rows high.
pub const HEIGHT: usize = 25;

/// The VGA text buffer is a two-dimensional array with typically 25 rows and 80 columns, which is
/// directly rendered to the screen.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; WIDTH]; HEIGHT],
}

/// A writer for writing to the VGA buffer, which is then rendered to the screen.
struct Writer {
    col_pos: usize,
    row_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Returns the color code.
    fn get_color_code(&self) -> (u8, u8) {
        (self.color_code.fg(), self.color_code.bg())
    }

    /// Sets the color code.
    fn set_color_code(&mut self, fg_color: Color, bg_color: Color) {
        self.color_code = ColorCode::new(fg_color, bg_color);
    }

    /// Returns data at the specified position from the VGA buffer.
    fn query_data_at(&self, row: usize, col: usize) -> Result<(u8, u8), &'static str> {
        match (row, col) {
            (0..HEIGHT, 0..WIDTH) => {
                let screen_char = self.buffer.chars[row][col].read();
                Ok((screen_char.ascii_char, screen_char.color_code.as_u8()))
            }
            _ => Err("coordinates out of bounds")
        }
    }

    /// Writes the given byte to the VGA buffer.
    fn write_byte(&mut self, byte: u8) {
        match byte {
            CHAR_NEWLINE => {
                self.newline();
            }
            CHAR_BACKSPACE => {
                self.backspace();
            }
            CHAR_TAB => {
                self.tab();
            }
            CHAR_CARRIAGE_RETURN => {
                self.carriage_return();
            }
            CHAR_FORM_FEED => {
                self.form_feed();
            }
            byte => {
                if self.col_pos >= WIDTH {
                    self.newline();
                }
                let row = self.row_pos;
                let col = self.col_pos;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: byte,
                    color_code,
                });
                self.col_pos += 1;
            }
        }
    }

    /// Writes the given string to the VGA buffer byte-by-byte.
    fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                RANGE_PRINTABLE_ASCII_BEGIN..=RANGE_PRINTABLE_ASCII_END |
                CHAR_NEWLINE |
                CHAR_BACKSPACE |
                CHAR_TAB |
                CHAR_FORM_FEED |
                CHAR_CARRIAGE_RETURN => {
                    self.write_byte(byte)
                }
                _ => {
                    self.write_byte(FALLBACK_CHAR)
                }
            }
        }
        self.update_cursor();
    }

    /// Uni-directionally scrolls the view.
    fn scroll_view(&mut self) {
        for row in 1..HEIGHT {
            for col in 0..WIDTH {
                let ch = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(ch);
            }
        }
        self.clear_row(HEIGHT - 1);
    }

    /// Outputs a new line.
    fn newline(&mut self) {
        if self.row_pos < (HEIGHT - 1) {
            self.row_pos += 1;
        } else {
            self.scroll_view();
        }
        self.col_pos = 0;
    }

    /// Outputs a backspace.
    fn backspace(&mut self) {
        if self.col_pos > 0 {
            self.col_pos -= 1;
            self.write_byte(CHAR_SPACE);
            self.col_pos -= 1;
        }
    }

    /// Outputs a tab.
    fn tab(&mut self) {
        for _ in 0..TAB_WIDTH {
            self.write_byte(CHAR_SPACE);
        }
    }

    /// Outputs a carriage return.
    fn carriage_return(&mut self) {
        self.col_pos = 0;
    }

    /// Outputs a form feed.
    fn form_feed(&mut self) {
        self.newline();
        self.write_byte(CHAR_SPACE);
    }

    /// Clears the given row.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: CHAR_SPACE,
            color_code: self.color_code,
        };
        for col in 0..WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Clears the whole screen.
    fn clear(&mut self) {
        for r in 0..HEIGHT {
            self.clear_row(r);
        }
        self.col_pos = 0;
        self.row_pos = 0;
        self.update_cursor();
    }

    /// Updates the cursor position.
    fn update_cursor(&mut self) {
        let cur_offset = (self.row_pos * WIDTH) + self.col_pos;

        // Control Address Register - 0x3D4
        let mut car = Port::new(0x3D4);
        // Horizontal Total Register - 0x3D5:0
        let mut htr = Port::new(0x3D5);
        unsafe {
            car.write(0x0Fu16);
            htr.write((cur_offset & 0xFF) as u16);
            car.write(0x0Eu16);
            htr.write(((cur_offset >> 8) & 0xFF) as u16);
        };
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

lazy_static! {
    // Global Interface for writing to VGA Buffer.
    static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row_pos: 0,
        col_pos: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(ADDRESS as *mut Buffer) },
    });
}

/// A secure public interface for retrieving the color of the foreground and background.
pub fn get_color_code() -> (u8, u8) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().get_color_code() }
    )
}

/// A secure public interface for changing the color of the foreground and background.
pub fn set_color_code(fg_color: Color, bg_color: Color) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().set_color_code(fg_color, bg_color); }
    );
}

/// A secure public interface for querying data at the specified position from the VGA buffer.
pub fn query_data_at(row: usize, col: usize) -> Result<(u8, u8), &'static str> {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().query_data_at(row, col) }
    )
}

/// Safe public interface for clearing the screen.
pub fn clear() {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().clear(); }
    );
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;

    instructions::interrupts::without_interrupts(
        || { WRITER.lock().write_fmt(args).unwrap(); }
    );
}

// Macros

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
