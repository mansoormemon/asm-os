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

use core::str::FromStr;

/////////////
// Globals
/////////////

/// Number of supported colors.
pub const TOTAL_COLORS: usize = 16;

/// List of colors.
pub const COLORS: [Color; TOTAL_COLORS] = [
    Color::Black,
    Color::Blue,
    Color::Green,
    Color::Cyan,
    Color::Red,
    Color::Magenta,
    Color::Brown,
    Color::LightGray,
    Color::DarkGray,
    Color::LightBlue,
    Color::LightGreen,
    Color::LightCyan,
    Color::LightRed,
    Color::Pink,
    Color::Yellow,
    Color::White
];

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
    /// Creates a new object from enum index.
    pub fn from_index(idx: u8) -> Result<Self, ()> {
        match idx {
            0x0 => Ok(Self::Black),
            0x1 => Ok(Self::Blue),
            0x2 => Ok(Self::Green),
            0x3 => Ok(Self::Cyan),
            0x4 => Ok(Self::Red),
            0x5 => Ok(Self::Magenta),
            0x6 => Ok(Self::Brown),
            0x7 => Ok(Self::LightGray),
            0x8 => Ok(Self::DarkGray),
            0x9 => Ok(Self::LightBlue),
            0xA => Ok(Self::LightGreen),
            0xB => Ok(Self::LightCyan),
            0xC => Ok(Self::LightRed),
            0xD => Ok(Self::Pink),
            0xE => Ok(Self::Yellow),
            0xF => Ok(Self::White),
            _ => Err(()),
        }
    }

    /// Creates a new object from ANSI code.
    pub fn from_ansi(code: u8) -> Result<Self, ()> {
        match code {
            30 => Ok(Self::Black),
            31 => Ok(Self::Red),
            32 => Ok(Self::Green),
            33 => Ok(Self::Brown),
            34 => Ok(Self::Blue),
            35 => Ok(Self::Magenta),
            36 => Ok(Self::Cyan),
            37 => Ok(Self::LightGray),
            90 => Ok(Self::DarkGray),
            91 => Ok(Self::LightRed),
            92 => Ok(Self::LightGreen),
            93 => Ok(Self::Yellow),
            94 => Ok(Self::LightBlue),
            95 => Ok(Self::Pink),
            96 => Ok(Self::LightCyan),
            97 => Ok(Self::White),
            _ => Err(()),
        }
    }

    /// Returns the object as an enum index.
    pub fn as_u8(&self) -> u8 { (*self) as u8 }

    /// Returns the object as a primitive string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Black => "black",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Cyan => "cyan",
            Self::Red => "red",
            Self::Magenta => "magenta",
            Self::Brown => "brown",
            Self::LightGray => "light-gray",
            Self::DarkGray => "dark-gray",
            Self::LightBlue => "light-blue",
            Self::LightGreen => "light-green",
            Self::LightCyan => "light-cyan",
            Self::LightRed => "light-red",
            Self::Pink => "pink",
            Self::Yellow => "yellow",
            Self::White => "white",
        }
    }

    /// Returns the corresponding ANSI code.
    pub fn to_ansi(&self) -> u8 {
        match self {
            Self::Black => 30,
            Self::Blue => 34,
            Self::Green => 32,
            Self::Cyan => 36,
            Self::Red => 31,
            Self::Magenta => 35,
            Self::Brown => 33,
            Self::LightGray => 37,
            Self::DarkGray => 90,
            Self::LightBlue => 94,
            Self::LightGreen => 92,
            Self::LightCyan => 96,
            Self::LightRed => 91,
            Self::Pink => 95,
            Self::Yellow => 93,
            Self::White => 97,
        }
    }

    /// Returns the associated VGA register.
    pub fn associated_vga_register(&self) -> u8 {
        match self {
            Self::Black => 0x00,
            Self::Blue => 0x01,
            Self::Green => 0x02,
            Self::Cyan => 0x03,
            Self::Red => 0x04,
            Self::Magenta => 0x05,
            Self::Brown => 0x14,
            Self::LightGray => 0x07,
            Self::DarkGray => 0x38,
            Self::LightBlue => 0x39,
            Self::LightGreen => 0x3A,
            Self::LightCyan => 0x3B,
            Self::LightRed => 0x3C,
            Self::Pink => 0x3D,
            Self::Yellow => 0x3E,
            Self::White => 0x3F,
        }
    }
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "black" => Ok(Self::Black),
            "blue" => Ok(Self::Blue),
            "green" => Ok(Self::Green),
            "cyan" => Ok(Self::Cyan),
            "red" => Ok(Self::Red),
            "magenta" => Ok(Self::Magenta),
            "brown" => Ok(Self::Brown),
            "light-gray" => Ok(Self::LightGray),
            "dark-gray" => Ok(Self::DarkGray),
            "light-blue" => Ok(Self::LightBlue),
            "light-green" => Ok(Self::LightGreen),
            "light-cyan" => Ok(Self::LightCyan),
            "light-red" => Ok(Self::LightRed),
            "pink" => Ok(Self::Pink),
            "yellow" => Ok(Self::Yellow),
            "white" => Ok(Self::White),
            _ => Err(()),
        }
    }
}
