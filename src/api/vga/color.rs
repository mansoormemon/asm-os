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

/////////////
/// Color
/////////////
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

impl Color {
    /// Creates a new object from the given index.
    pub fn from_index(index: usize) -> Color {
        match index {
            0x0 => Color::Black,
            0x1 => Color::Blue,
            0x2 => Color::Green,
            0x3 => Color::Cyan,
            0x4 => Color::Red,
            0x5 => Color::Magenta,
            0x6 => Color::Brown,
            0x7 => Color::LightGray,
            0x8 => Color::DarkGray,
            0x9 => Color::LightBlue,
            0xA => Color::LightGreen,
            0xB => Color::LightCyan,
            0xC => Color::LightRed,
            0xD => Color::Pink,
            0xE => Color::Yellow,
            0xF => Color::White,
            _ => Color::Black,
        }
    }

    /// Creates a new object from the given ANSI code.
    pub fn from_ansi(code: u8) -> Color {
        match code {
            30 => Color::Black,
            31 => Color::Red,
            32 => Color::Green,
            33 => Color::Brown,
            34 => Color::Blue,
            35 => Color::Magenta,
            36 => Color::Cyan,
            37 => Color::LightGray,
            90 => Color::DarkGray,
            91 => Color::LightRed,
            92 => Color::LightGreen,
            93 => Color::Yellow,
            94 => Color::LightBlue,
            95 => Color::Pink,
            96 => Color::LightCyan,
            97 => Color::White,
            _ => Color::Black,
        }
    }

    /// Returns the corresponding ANSI code.
    pub fn to_ansi(&self) -> u8 {
        match self {
            Color::Black => 30,
            Color::Blue => 34,
            Color::Green => 32,
            Color::Cyan => 36,
            Color::Red => 31,
            Color::Magenta => 35,
            Color::Brown => 33,
            Color::LightGray => 37,
            Color::DarkGray => 90,
            Color::LightBlue => 94,
            Color::LightGreen => 92,
            Color::LightCyan => 96,
            Color::LightRed => 91,
            Color::Pink => 95,
            Color::Yellow => 93,
            Color::White => 97,
        }
    }

    /// Returns the associated VGA register.
    pub fn to_vga_register(&self) -> u8 {
        match self {
            Color::Black => 0x00,
            Color::Blue => 0x01,
            Color::Green => 0x02,
            Color::Cyan => 0x03,
            Color::Red => 0x04,
            Color::Magenta => 0x05,
            Color::Brown => 0x14,
            Color::LightGray => 0x07,
            Color::DarkGray => 0x38,
            Color::LightBlue => 0x39,
            Color::LightGreen => 0x3A,
            Color::LightCyan => 0x3B,
            Color::LightRed => 0x3C,
            Color::Pink => 0x3D,
            Color::Yellow => 0x3E,
            Color::White => 0x3F,
        }
    }
}
