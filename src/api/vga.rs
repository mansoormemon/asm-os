use x86_64::instructions;

use crate::api::vga::color::Color;
use crate::api::vga::font::Font;
use crate::api::vga::palette::Palette;
use crate::kernel;
use crate::kernel::error::GenericError;
use crate::kernel::vga::WRITER;

pub mod color;
pub mod cursor;
pub mod font;
pub mod palette;

//////////////
// Defaults //
//////////////

pub struct Default;

impl Default {
    pub const FOREGROUND: Color = Color::LightGray;
    pub const BACKGROUND: Color = Color::Black;
    pub const TAB_WIDTH: u8 = 8;
    pub const CURSOR_ENABLED: bool = true;
    pub const CURSOR_STYLE: cursor::Style = cursor::Style::Block;
}

/// Returns the width of the VGA buffer.
pub fn width() -> usize {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().width() }
    )
}

/// Returns the height of the VGA buffer.
pub fn height() -> usize {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().height() }
    )
}

/// Returns the cursor's position.
pub fn get_cursor_pos() -> (usize, usize) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().get_cursor_pos() }
    )
}

/// Moves the cursor to the specified position.
pub fn set_cursor_pos(row: usize, col: usize) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().set_cursor_pos(row, col); }
    );
}

/// Returns the current foreground color.
pub fn get_foreground() -> Color {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().get_foreground() }
    )
}

/// Sets the foreground color.
pub fn set_foreground(fg: Color) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().set_foreground(fg); }
    );
}

/// Resets the foreground color.
pub fn reset_foreground() {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().reset_foreground(); }
    );
}

/// Returns the current background color.
pub fn get_background() -> Color {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().get_background() }
    )
}

/// Sets the background color.
pub fn set_background(bg: Color) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().set_background(bg); }
    );
}

/// Resets the background colour.
pub fn reset_background() {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().reset_background(); }
    );
}

/// Retrieve the color of the foreground and background.
pub fn get_color_code() -> (Color, Color) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().get_color_code() }
    )
}

/// Set the color of the foreground and background.
pub fn set_color_code(fg: Color, bg: Color) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().set_color_code(fg, bg); }
    );
}

/// Resets the color of the foreground and background.
pub fn reset_color_code() {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().reset_color_code(); }
    );
}

/// Returns data at the specified position from the VGA buffer.
pub fn query_data_at(row: usize, col: usize) -> Result<(u8, u8), GenericError> {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().query_data_at(row, col) }
    )
}

/// Sets the VGA color palette.
pub fn set_palette(palette: Palette) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().set_palette(palette); }
    );
}

/// Sets the VGA font.
pub fn set_font(font: &Font) {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().set_font(&font); }
    );
}

/// Clears the screen.
pub fn clear() {
    instructions::interrupts::without_interrupts(
        || { WRITER.lock().clear(); }
    );
}

pub fn is_cursor_enabled() -> bool {
    kernel::vga::is_cursor_enabled()
}

/// Enables the cursor.
pub fn enable_cursor() {
    kernel::vga::enable_cursor();
}

/// Disables the cursor.
pub fn disable_cursor() {
    kernel::vga::disable_cursor();
}

/// Returns the current tab width.
pub fn get_tab_width() -> u8 {
    kernel::vga::get_tab_width()
}

/// Sets tab width.
pub fn set_tab_width(tab_width: u8) {
    kernel::vga::set_tab_width(tab_width);
}

/// Resets the tab width.
pub fn reset_tab_width() {
    kernel::vga::reset_tab_width();
}

/// Returns the current cursor style.
pub fn get_cursor_style() -> cursor::Style {
    kernel::vga::get_cursor_style()
}

/// Sets the cursor style.
pub fn set_cursor_style(cursor_style: cursor::Style) {
    kernel::vga::set_cursor_style(cursor_style);
}

/// Resets the cursor style.
pub fn reset_cursor_style() {
    kernel::vga::reset_cursor_style();
}

/// Sets the location for the underline.
pub fn set_underline_location(location: u8) {
    kernel::vga::set_underline_location(location);
}
