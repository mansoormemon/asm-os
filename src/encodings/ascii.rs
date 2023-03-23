// MIT License
//
// Copyright (c) 2023 Mansoor Ahmed Memon.
//
// Permission is hereby granted, free of u8ge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, lish, distribute, sublicense, and/or sell
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

use super::Charset;

//////////////////////////////////////////////////////////////////
/// American Standard Code for Information Interchange (ASCII)
//////////////////////////////////////////////////////////////////
pub struct ASCII<T> {
    __unused: T,
}

impl Charset<u8> for ASCII<u8> {
    const NUL: u8 = 0x00;
    const SOH: u8 = 0x01;
    const STX: u8 = 0x02;
    const ETX: u8 = 0x03;
    const EOT: u8 = 0x04;
    const ENQ: u8 = 0x05;
    const ACK: u8 = 0x06;
    const BEL: u8 = 0x07;
    const BS: u8 = 0x08;
    const HT: u8 = 0x09;
    const LF: u8 = 0x0A;
    const VT: u8 = 0x0B;
    const FF: u8 = 0x0C;
    const CR: u8 = 0x0D;
    const SO: u8 = 0x0E;
    const SI: u8 = 0x0F;
    const DLE: u8 = 0x10;
    const DC1: u8 = 0x11;
    const DC2: u8 = 0x12;
    const DC3: u8 = 0x13;
    const DC4: u8 = 0x14;
    const NAK: u8 = 0x15;
    const SYN: u8 = 0x16;
    const ETB: u8 = 0x17;
    const CAN: u8 = 0x18;
    const EM: u8 = 0x19;
    const SUB: u8 = 0x1A;
    const ESC: u8 = 0x1B;
    const FS: u8 = 0x1C;
    const GS: u8 = 0x1D;
    const RS: u8 = 0x1E;
    const US: u8 = 0x1F;
    const SP: u8 = 0x20;
    const DEL: u8 = 0x7F;
}

impl Charset<char> for ASCII<char> {
    const NUL: char = '\x00';
    const SOH: char = '\x01';
    const STX: char = '\x02';
    const ETX: char = '\x03';
    const EOT: char = '\x04';
    const ENQ: char = '\x05';
    const ACK: char = '\x06';
    const BEL: char = '\x07';
    const BS: char = '\x08';
    const HT: char = '\x09';
    const LF: char = '\x0A';
    const VT: char = '\x0B';
    const FF: char = '\x0C';
    const CR: char = '\x0D';
    const SO: char = '\x0E';
    const SI: char = '\x0F';
    const DLE: char = '\x10';
    const DC1: char = '\x11';
    const DC2: char = '\x12';
    const DC3: char = '\x13';
    const DC4: char = '\x14';
    const NAK: char = '\x15';
    const SYN: char = '\x16';
    const ETB: char = '\x17';
    const CAN: char = '\x18';
    const EM: char = '\x19';
    const SUB: char = '\x1A';
    const ESC: char = '\x1B';
    const FS: char = '\x1C';
    const GS: char = '\x1D';
    const RS: char = '\x1E';
    const US: char = '\x1F';
    const SP: char = '\x20';
    const DEL: char = '\x7F';
}
