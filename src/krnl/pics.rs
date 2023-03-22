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

use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions;

use crate::success;

////////////////
// Attributes
////////////////

/// Offset of Master PIC.
pub const M_OFFSET: u8 = 32;
/// Pins in Master PIC.
pub const M_PIN_COUNT: u8 = 8;
/// Data port of Master PIC.
pub const M_DATA_PORT: u16 = 0x21;

/// Offset of Slave PIC.
pub const S_OFFSET: u8 = M_OFFSET + M_PIN_COUNT;
/// Pins in Slave PIC.
pub const S_PIN_COUNT: u8 = 8;
/// Data port of Slave PIC.
pub const S_DATA_PORT: u16 = 0xA1;

/// Total pins.
pub const TOTAL_PIN_COUNT: u8 = M_PIN_COUNT + S_PIN_COUNT;

/////////////
// Mutexes
/////////////

/// 8259 Programmable Interrupt Controller (PIC)
///
/// The 8259 Programmable Interrupt Controller (PIC) is one of the most important chips making up
/// the x86 architecture. Without it, the x86 architecture would not be an interrupt driven
/// architecture. The function of the 8259A is to manage hardware interrupts and send them to the
/// appropriate system interrupt. This allows the system to respond to devices needs without loss
/// of time.
///
/// OS Dev Wiki: https://wiki.osdev.org/8259_PIC
pub(crate) static PICS_8259: Mutex<ChainedPics> = Mutex::new(
    unsafe { ChainedPics::new(M_OFFSET, S_OFFSET) }
);

///////////////
// Utilities
///////////////

/// Initializes the PICs.
pub(crate) fn init() {
    unsafe {
        PICS_8259.lock().initialize();
    }
    success!("PICs initialized");
}

/// Enables interrupts.
pub(crate) fn enable() {
    instructions::interrupts::enable();
    success!("PICs enabled");
}
