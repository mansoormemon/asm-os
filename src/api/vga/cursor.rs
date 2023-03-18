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

/// Number of supported styles.
pub const TOTAL_STYLES: usize = 2;

// List of styles.
pub const STYLES: [Style; TOTAL_STYLES] = [
    Style::Underscore,
    Style::Block
];

/////////////
/// Style
/////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Style {
    Underscore = 0x0,
    Block = 0x1,
}

impl Style {
    /// Creates a new object from enum index.
    pub fn from_index(idx: u8) -> Result<Self, ()> {
        match idx {
            0x0 => Ok(Self::Underscore),
            0x1 => Ok(Self::Block),
            _ => Err(()),
        }
    }

    /// Returns the object as an enum index.
    pub fn as_u8(&self) -> u8 { (*self) as u8 }

    /// Returns the object as a primitive string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Underscore => "underscore",
            Self::Block => "block",
        }
    }

    /// Returns the scanline bounds.
    pub fn scanline_bounds(&self) -> (u8, u8) {
        match self {
            Self::Underscore => (0xD, 0xE),
            Self::Block => (0x1, 0xE),
        }
    }
}

impl FromStr for Style {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "underscore" => Ok(Self::Underscore),
            "block" => Ok(Self::Block),
            _ => Err(())
        }
    }
}
