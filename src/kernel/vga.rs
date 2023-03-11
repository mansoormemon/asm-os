// Video Graphics Array (VGA)
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

use core::cmp::min;
use core::fmt;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use vte::{Parser, Perform};
use x86_64::instructions;
use x86_64::instructions::port::Port;

use crate::api::char;
use crate::api::vga::{clear, cursor, Default, palette};
use crate::api::vga::color::Color;
use crate::api::vga::font::Font;
use crate::api::vga::palette::Palette;
use crate::kernel::error::GenericError;

///////////////////////
// Global Interfaces //
///////////////////////

lazy_static! {
    /// A global interface for VGA buffer writer.
    pub(crate) static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}

//////////////////////
// Local Interfaces //
//////////////////////

lazy_static! {
    /// A global interface for ANSI parser.
    static ref PARSER: Mutex<Parser> = Mutex::new(Parser::new());
}

////////////////////
// Configurations //
////////////////////

/// Tab width.
static TAB_WIDTH: AtomicU8 = AtomicU8::new(Default::TAB_WIDTH);

/// Cursor enabled.
static CURSOR_ENABLED: AtomicBool = AtomicBool::new(Default::CURSOR_ENABLED);

/// Cursor style.
static CURSOR_STYLE: AtomicU8 = AtomicU8::new(Default::CURSOR_STYLE as u8);

///////////////////////
// Buffer Attributes //
///////////////////////

/// The VGA text buffer can be accessed via memory mapped at 0xB8000.
const TEXT_BUFFER: isize = 0xB8000;
/// The VGA graphics buffer can be accessed via memory mapped at 0xA0000.
const GRAPHICS_BUFFER: isize = 0xA0000;
/// The VGA text buffer is typically 80 columns wide.
const WIDTH: usize = 80;
/// The VGA text buffer is typically 25 rows high.
const HEIGHT: usize = 25;

////////////////
/// Register ///
////////////////
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
enum Register {
    /// Attribute Address Register.
    AttrAddr = 0x3C0,
    /// Attribute Data Register.
    AttrData = 0x3C1,
    /// Sequence Memory Mode Register.
    SequencerAddr = 0x3C4,
    /// DAC Address Register.
    DACAddr = 0x3C8,
    /// DAC Data Register.
    DACData = 0x3C9,
    /// Graphics Address Register.
    GraphicsAddr = 0x3CE,
    /// CRT Control Address Register.
    CRTControlAddr = 0x3D4,
    /// CRT Control Data Register.
    CRTControlData = 0x3D5,
    /// Input Status Register.
    InputStatus = 0x3DA,
}

//////////////////
/// Color Code ///
//////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Creates a new color code from the given params.
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }

    /// Extracts the foreground color from the color code.
    fn get_fg(&self) -> u8 {
        self.0 & 0xF
    }

    /// Overwrites the bits representing the foreground color in the color code.
    fn set_fg(&mut self, fg: Color) {
        self.0 = (self.0 & 0xF0) | (fg as u8)
    }

    /// Extracts the background color from the color code.
    fn get_bg(&self) -> u8 {
        self.0 >> 4
    }

    /// Overwrites the bits representing the background color in the color code.
    fn set_bg(&mut self, bg: Color) {
        self.0 = ((bg as u8) << 4) | (self.0 & 0xF)
    }

    /// Returns the color code represented as a `u8`.
    fn as_u8(&self) -> u8 {
        self.0
    }
}

////////////////////////
/// Screen Character ///
////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

//////////////
/// Buffer ///
//////////////
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; WIDTH]; HEIGHT],
}

//////////////
/// Writer ///
//////////////
pub(crate) struct Writer {
    col_pos: usize,
    row_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Creates a new object.
    fn new() -> Self {
        Writer {
            row_pos: 0,
            col_pos: 0,
            color_code: ColorCode::new(Default::FOREGROUND, Default::BACKGROUND),
            buffer: unsafe { &mut *(TEXT_BUFFER as *mut Buffer) }
        }
    }

    /// Returns the width of the VGA buffer.
    pub(crate) fn width(&self) -> usize { WIDTH }

    /// Returns the height of the VGA buffer.
    pub(crate) fn height(&self) -> usize { HEIGHT }

    /// Returns the cursor's position.
    pub(crate) fn get_cursor_pos(&self) -> (usize, usize) {
        (self.row_pos, self.col_pos)
    }

    /// Sets the cursor to the specified position.
    pub(crate) fn set_cursor_pos(&mut self, row: usize, col: usize) {
        self.row_pos = min(row, HEIGHT - 1);
        self.col_pos = min(col, WIDTH - 1);
        self.update_cursor();
    }

    /// Returns the current foreground color.
    pub(crate) fn get_foreground(&self) -> Color {
        Color::from_index(self.color_code.get_fg() as usize)
    }

    /// Sets the foreground color.
    pub(crate) fn set_foreground(&mut self, fg: Color) {
        self.color_code.set_fg(fg);
    }

    /// Resets the foreground color.
    pub(crate) fn reset_foreground(&mut self) {
        self.set_foreground(Default::FOREGROUND);
    }

    /// Returns the current background color.
    pub(crate) fn get_background(&self) -> Color {
        Color::from_index(self.color_code.get_bg() as usize)
    }

    /// Sets the background color.
    pub(crate) fn set_background(&mut self, bg: Color) {
        self.color_code.set_bg(bg);
    }

    /// Resets the background color.
    pub(crate) fn reset_background(&mut self) {
        self.set_background(Default::BACKGROUND);
    }

    /// Retrieve the color of the foreground and background.
    pub(crate) fn get_color_code(&self) -> (Color, Color) {
        (self.get_foreground(), self.get_background())
    }

    /// Set the color of the foreground and background.
    pub(crate) fn set_color_code(&mut self, fg: Color, bg: Color) {
        self.color_code = ColorCode::new(fg, bg);
    }

    /// Resets the color of the foreground and background.
    pub(crate) fn reset_color_code(&mut self) {
        self.color_code = ColorCode::new(Default::FOREGROUND, Default::BACKGROUND);
    }

    /// Returns data at the specified position from the VGA buffer.
    pub(crate) fn query_data_at(&self, row: usize, col: usize) -> Result<(u8, u8), GenericError> {
        match (row, col) {
            (0..HEIGHT, 0..WIDTH) => {
                let screen_char = self.buffer.chars[row][col].read();
                Ok((screen_char.ascii_char, screen_char.color_code.as_u8()))
            }
            _ => Err(GenericError::IndexOutOfBounds)
        }
    }

    /// Updates the cursor position.
    fn update_cursor(&mut self) {
        let mut car = Port::new(Register::CRTControlAddr as u16);
        let mut cdr = Port::new(Register::CRTControlData as u16);
        let cur_offset = (self.row_pos * WIDTH) + self.col_pos;
        unsafe {
            car.write(0x0Fu16);
            cdr.write((cur_offset & 0xFF) as u16);
            car.write(0x0Eu16);
            cdr.write(((cur_offset >> 8) & 0xFF) as u16);
        };
    }

    /// Sets the VGA color palette.
    pub(crate) fn set_palette(&mut self, palette: Palette) {
        const CONTRAST: u8 = 2;

        let mut addr = Port::new(Register::DACAddr as u16);
        let mut data = Port::new(Register::DACData as u16);

        let vga_color = |color: u8| -> u8 { color >> CONTRAST };
        for (i, (r, g, b)) in palette.colors.iter().enumerate() {
            let reg = Color::from_index(i).to_vga_register();
            unsafe {
                addr.write(reg);
                data.write(vga_color(*r));
                data.write(vga_color(*g));
                data.write(vga_color(*b));
            }
        }
    }

    /// Sets the VGA font.
    pub(crate) fn set_font(&mut self, font: &Font) {
        const BUFFER: *mut u8 = GRAPHICS_BUFFER as *mut u8;
        const CHAR_BYTE_BOUNDARY: u8 = 32;

        let mut sequencer: Port<u16> = Port::new(Register::SequencerAddr as u16);
        let mut graphics: Port<u16> = Port::new(Register::GraphicsAddr as u16);

        unsafe {
            sequencer.write(0x0100); // Do a sy/**/nc reset.
            sequencer.write(0x0402); // Write to plane 2 only.
            sequencer.write(0x0704); // Sequential access.
            sequencer.write(0x0300); // End the reset.
            graphics.write(0x0204); // Read from plane 2 only.
            graphics.write(0x0005); // Disable odd/even.
            graphics.write(0x0006); // VRAM at 0xA0000.

            for i in 0..font.size as usize {
                for j in 0..font.height as usize {
                    let vga_offset = j + i * CHAR_BYTE_BOUNDARY as usize;
                    let fnt_offset = j + i * font.height as usize;
                    BUFFER.add(vga_offset).write_volatile(font.data[fnt_offset]);
                }
            }

            sequencer.write(0x0100); // Do a sync reset.
            sequencer.write(0x0302); // Write to plane 0 & 1.
            sequencer.write(0x0304); // Even/odd access.
            sequencer.write(0x0300); // End the reset.
            graphics.write(0x0004); // Restore to default.
            graphics.write(0x1005); // Resume odd/even.
            graphics.write(0x0E06); // VRAM at 0xB8000.
        }
    }

    /// Writes the given byte to the VGA buffer.
    fn write_byte(&mut self, byte: u8) {
        match byte {
            char::NEWLINE => {
                self.newline();
            }
            char::BACKSPACE => {
                self.backspace();
            }
            char::HTAB => {
                self.htab();
            }
            char::CARRIAGE_RETURN => {
                self.carriage_return();
            }
            char::FORM_FEED => {
                self.form_feed();
            }
            byte => {
                if self.col_pos >= self.width() {
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
            self.write_byte(char::SPACE);
            self.col_pos -= 1;
        }
    }

    /// Outputs a tab.
    fn htab(&mut self) {
        let tab_width = TAB_WIDTH.load(Ordering::Relaxed);
        for _ in 0..tab_width as usize {
            self.write_byte(char::SPACE);
        }
    }

    /// Outputs a carriage return.
    fn carriage_return(&mut self) {
        self.col_pos = 0;
    }

    /// Outputs a form feed.
    fn form_feed(&mut self) {
        self.newline();
        self.write_byte(char::SPACE);
    }

    /// Clears the given row.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: char::SPACE,
            color_code: self.color_code,
        };
        for col in 0..WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Clears the whole screen.
    pub(crate) fn clear(&mut self) {
        for r in 0..HEIGHT {
            self.clear_row(r);
        }
        self.col_pos = 0;
        self.row_pos = 0;
        self.update_cursor();
    }
}

impl Perform for Writer {
    fn print(&mut self, c: char) {
        self.write_byte(c as u8);
    }

    fn execute(&mut self, byte: u8) {
        self.write_byte(byte);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut parser = PARSER.lock();
        for byte in s.bytes() {
            parser.advance(self, byte);
        }
        self.update_cursor();
        Ok(())
    }
}

/// Returns the value stored in Attribute Address Data Register at specified index.
fn get_attr_ctrl_reg(index: u8) -> u8 {
    const PALETTE_ADDR_SOURCE_MASK: u8 = 0x20;
    instructions::interrupts::without_interrupts(
        || {
            let mut isr: Port<u8> = Port::new(Register::InputStatus as u16);
            let mut addr: Port<u8> = Port::new(Register::AttrAddr as u16);
            let mut data: Port<u8> = Port::new(Register::AttrData as u16);

            unsafe {
                isr.read();
                let byte = addr.read();
                addr.write(index | PALETTE_ADDR_SOURCE_MASK);
                let res = data.read();
                addr.write(byte);
                res
            }
        }
    )
}

/// Sets the value of Attribute Address Register at specified index.
fn set_attr_ctrl_reg(index: u8, value: u8) {
    instructions::interrupts::without_interrupts(
        || {
            let mut isr: Port<u8> = Port::new(Register::InputStatus as u16);
            let mut addr: Port<u8> = Port::new(Register::AttrAddr as u16);

            unsafe {
                isr.read();
                let byte = addr.read();
                addr.write(index);
                addr.write(value);
                addr.write(byte);
            }
        }
    )
}

/// Returns whether the cursor is enabled or not.
pub(crate) fn is_cursor_enabled() -> bool {
    CURSOR_ENABLED.load(Ordering::Relaxed)
}

/// Enables the cursor.
pub(crate) fn enable_cursor() {
    const REG_CURSOR_START: u8 = 0x0A;
    const REG_CURSOR_END: u8 = 0x0B;

    let mut addr: Port<u8> = Port::new(Register::CRTControlAddr as u16);
    let mut data: Port<u8> = Port::new(Register::CRTControlData as u16);

    let (scanline_begin, scanline_end) = cursor::Style::from_index(
        CURSOR_STYLE.load(Ordering::Relaxed) as usize
    ).get_scanline_bounds();
    unsafe {
        addr.write(REG_CURSOR_START);
        let byte = data.read();
        data.write((byte & 0xC0) | scanline_begin);

        addr.write(REG_CURSOR_END);
        let byte = data.read();
        data.write((byte & 0xE0) | scanline_end);
    }
    CURSOR_ENABLED.store(true, Ordering::Relaxed)
}

/// Disables the cursor.
pub(crate) fn disable_cursor() {
    let mut addr = Port::new(Register::CRTControlAddr as u16);
    let mut data = Port::new(Register::CRTControlData as u16);
    unsafe {
        addr.write(0x0Au8);
        data.write(0x20u8);
    }
    CURSOR_ENABLED.store(false, Ordering::Relaxed)
}

/// Returns the current tab width.
pub(crate) fn get_tab_width() -> u8 {
    TAB_WIDTH.load(Ordering::Relaxed)
}

/// Sets tab width.
pub(crate) fn set_tab_width(tab_width: u8) {
    if tab_width > 0 {
        TAB_WIDTH.store(tab_width, Ordering::Relaxed);
    }
}

/// Resets the tab width.
pub(crate) fn reset_tab_width() {
    TAB_WIDTH.store(Default::TAB_WIDTH, Ordering::Relaxed);
}

/// Returns the current cursor style.
pub(crate) fn get_cursor_style() -> cursor::Style {
    cursor::Style::from_index(CURSOR_STYLE.load(Ordering::Relaxed) as usize)
}

/// Sets the cursor style.
pub(crate) fn set_cursor_style(cursor_style: cursor::Style) {
    CURSOR_STYLE.store(cursor_style as u8, Ordering::Relaxed);
    if is_cursor_enabled() {
        enable_cursor();
    }
}

/// Resets the cursor style.
pub(crate) fn reset_cursor_style() {
    CURSOR_STYLE.store(Default::CURSOR_STYLE as u8, Ordering::Relaxed);
}

/// Sets the underline location.
pub(crate) fn set_underline_location(location: u8) {
    const REG_UNDERLINE_LOC: u8 = 0x14;

    instructions::interrupts::without_interrupts(
        || {
            let mut addr: Port<u8> = Port::new(Register::CRTControlAddr as u16);
            let mut data: Port<u8> = Port::new(Register::CRTControlData as u16);

            unsafe {
                addr.write(REG_UNDERLINE_LOC);
                data.write(location);
            }
        }
    );
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;

    instructions::interrupts::without_interrupts(
        || { WRITER.lock().write_fmt(args).unwrap(); }
    );
}

////////////
// Macros //
////////////

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Initializes the VGA.
pub(crate) fn init() {
    // Map VGA color palette registers.
    for color in palette::COLORS.iter() {
        set_attr_ctrl_reg(*color as u8, color.to_vga_register());
    }

    // Clear blinking bit.
    const REG_ATTR_MODE_CTRL: u8 = 0x10;
    const MASK: u8 = 0xF7;
    let attr = get_attr_ctrl_reg(REG_ATTR_MODE_CTRL);
    set_attr_ctrl_reg(REG_ATTR_MODE_CTRL, attr & MASK);

    // Set location of underline.
    const BOTTOM_SC: u8 = 0xF;
    set_underline_location(BOTTOM_SC);

    // Enable the cursor.
    if is_cursor_enabled() {
        enable_cursor();
    }

    // Clear the screen.
    clear();
}
