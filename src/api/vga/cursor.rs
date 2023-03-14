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
/// Style
/////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Style {
    Underscore = 0x0,
    Block = 0x1,
}

impl Style {
    /// Creates a new object from the given index.
    pub fn from_index(index: usize) -> Self {
        match index {
            0x0 => Style::Underscore,
            0x1 => Style::Block,
            _ => Style::Underscore,
        }
    }

    /// Returns the scanline bounds.
    pub fn scanline_bounds(&self) -> (u8, u8) {
        match self {
            Style::Underscore => (0xD, 0xE),
            Style::Block => (0x1, 0xE),
        }
    }
}
