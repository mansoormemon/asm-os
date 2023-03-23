// MIT License
//
// Copyright (c) 2023 Mansoor Ahmed Memon.
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

use crate::drivers;

///////////////
/// Default
///////////////
pub struct Default;

impl Default {
    pub const LAYOUT: Layout = Layout::QWERTY;
}

//////////////
/// Layout
//////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Layout {
    AZERTY = 0x0,
    Dvorak = 0x1,
    QWERTY = 0x2,
}

impl Layout {
    /// Creates a new object from enum index.
    pub fn from_index(idx: u8) -> Result<Self, ()> {
        match idx {
            0x0 => Ok(Self::AZERTY),
            0x1 => Ok(Self::Dvorak),
            0x2 => Ok(Self::QWERTY),
            _ => Err(()),
        }
    }

    /// Returns the object as an enum index.
    pub fn as_u8(&self) -> u8 { (*self) as u8 }

    /// Returns the object as a primitive string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AZERTY => "azerty",
            Self::Dvorak => "dvorak",
            Self::QWERTY => "qwerty",
        }
    }
}

impl FromStr for Layout {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "azerty" => Ok(Self::AZERTY),
            "dvorak" => Ok(Self::Dvorak),
            "qwerty" => Ok(Self::QWERTY),
            _ => Err(())
        }
    }
}

/// Returns the layout.
pub fn get_layout() -> Layout { drivers::keyboard::get_layout() }

/// Sets the layout.
pub fn set_layout(lyt: Layout) { drivers::keyboard::set_layout(lyt); }

/// Resets the layout.
pub fn reset_layout() { drivers::keyboard::reset_layout(); }
