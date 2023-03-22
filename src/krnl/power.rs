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

/////////////////
// Utilities
/////////////////

use core::arch::asm;
use core::sync::atomic::Ordering;
use x86_64::instructions::port::Port;
use crate::krnl::acpi::{dsdt, fadt};

/// Shuts down the machine.
pub(crate) fn shutdown() {
    // todo: proper data encapsulation variables.
    let pm_1a_cnt_blk = fadt::PM_1A_CONTROL_BLOCK.load(Ordering::Relaxed);
    let slp_typa = dsdt::SLP_TYPA.load(Ordering::Relaxed);

    let mut port_pm_1a = Port::new(pm_1a_cnt_blk as u16);
    unsafe {
        port_pm_1a.write(slp_typa | dsdt::SLP_EN);
    }
}

pub fn reboot() {
    unsafe {
        asm!(
        "xor rax, rax",
        "mov cr3, rax",
        );
    }
}
