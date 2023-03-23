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

///////////////
/// Charset
///////////////
pub trait Charset<T> {
    const NUL: T;
    const SOH: T;
    const STX: T;
    const ETX: T;
    const EOT: T;
    const ENQ: T;
    const ACK: T;
    const BEL: T;
    const BS: T;
    const HT: T;
    const LF: T;
    const VT: T;
    const FF: T;
    const CR: T;
    const SO: T;
    const SI: T;
    const DLE: T;
    const DC1: T;
    const DC2: T;
    const DC3: T;
    const DC4: T;
    const NAK: T;
    const SYN: T;
    const ETB: T;
    const CAN: T;
    const EM: T;
    const SUB: T;
    const ESC: T;
    const FS: T;
    const GS: T;
    const RS: T;
    const US: T;
    const SP: T;
    const DEL: T;
}
