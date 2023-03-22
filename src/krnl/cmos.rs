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

use core::hint::spin_loop;

use x86_64::instructions;
use x86_64::instructions::port::Port;

////////////////////
// Configurations
////////////////////

/// Current century.
const RTC_CENTURY: u16 = 2000;

/////////////////////////////
/// Real-Time Clock (RTC)
/////////////////////////////
///
/// OS Dev Wiki: https://wiki.osdev.org/RTC
#[derive(PartialEq, Eq)]
pub struct RTC {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl RTC {
    /// Creates a new object.
    pub fn new() -> Self { CMOS::new().rtc() }

    /// Syncs with the CMOS chip.
    pub fn sync(&mut self) { *self = RTC::new(); }
}

///////////////////////
/// Register (CMOS)
///////////////////////
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Register {
    Second = 0x00,
    Minute = 0x02,
    Hour = 0x04,
    Day = 0x07,
    Month = 0x08,
    Year = 0x09,
    A = 0x0A,
    B = 0x0B,
    C = 0x0C,
}

/////////////////////////
/// Interrupt (CMOS)
/////////////////////////
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Interrupt {
    Periodic = 0x40,
    Alarm = 0x20,
    Update = 0x10,
}

//////////////////////////////////////////////////////
/// Complementary Metal-Oxide Semiconductor (CMOS)
//////////////////////////////////////////////////////
///
/// Complementary Metal-Oxide Semiconductor (CMOS)
///
/// CMOS is a tiny bit of very low power static memory that lives on the same chip as the
/// Real-Time Clock (RTC). It was introduced to IBM PC AT in 1984 which used Motorola MC146818A RTC.
///
/// CMOS (and the RTC) can only be accessed through IO Ports 0x70 and 0x71.
///
/// The function of the CMOS memory is to store 50 (or 114) bytes of "Setup" information for the BIOS
/// while the computer is turned off -- because there is a separate battery that keeps the Clock and
/// the CMOS information active.
///
/// Reference: https://stanislavs.org/helppc/cmos_ram.html
pub struct CMOS {
    addr: Port<u8>,
    data: Port<u8>,
}

impl CMOS {
    /// Creates a new object.
    pub fn new() -> Self {
        const ADDR_PORT: u16 = 0x70;
        const DATA_PORT: u16 = 0x71;

        CMOS {
            addr: Port::new(ADDR_PORT),
            data: Port::new(DATA_PORT),
        }
    }

    /// Returns a raw Real-Time Clock (RTC).
    fn rtc_raw(&mut self) -> RTC {
        RTC {
            second: self.read_register(Register::Second),
            minute: self.read_register(Register::Minute),
            hour: self.read_register(Register::Hour),
            day: self.read_register(Register::Day),
            month: self.read_register(Register::Month),
            year: self.read_register(Register::Year) as u16,
        }
    }

    /// Returns a parsed Real-Time Clock (RTC).
    pub fn rtc(&mut self) -> RTC {
        const SRB_BCD_MODE: u8 = 0x04;
        const SRB_H12_MODE: u8 = 0x02;

        const HOUR_H12_FMT: u8 = 0x80;

        // OS Dev Wiki: https://wiki.osdev.org/CMOS#Format_of_Bytes
        let bcd_to_binary = |bcd| -> u8 { ((bcd & 0xF0) >> 1) + ((bcd & 0xF0) >> 3) + (bcd & 0x0F) };
        let h12_to_h24 = |h12| -> u8 { ((h12 & 0x7F) + 12) % 24 };

        let mut rtc;

        // OS Dev Wiki: https://wiki.osdev.org/CMOS#RTC_Update_In_Progress
        loop {
            self.wait_while_updating();
            rtc = self.rtc_raw();
            self.wait_while_updating();
            if rtc == self.rtc_raw() { break; }
        }

        let status_reg_b = self.read_register(Register::B);

        // Convert BCD to binary.
        if status_reg_b & SRB_BCD_MODE == 0 {
            rtc.second = bcd_to_binary(rtc.second);
            rtc.minute = bcd_to_binary(rtc.minute);
            rtc.hour = bcd_to_binary(rtc.hour);
            rtc.day = bcd_to_binary(rtc.day);
            rtc.month = bcd_to_binary(rtc.month);
            rtc.year = bcd_to_binary(rtc.year as u8) as u16;
        }

        // Convert 12H to 24H.
        if (status_reg_b & SRB_H12_MODE == 0) && (rtc.hour & HOUR_H12_FMT == 0) { rtc.hour = h12_to_h24(rtc.hour); }

        // Add century.
        rtc.year += RTC_CENTURY;

        rtc
    }

    /// Sets the periodic interrupt rate.
    ///
    /// Note: `rate` must be above 2 and not over 15.
    pub fn set_periodic_interrupt_rate(&mut self, rate: u8) {
        instructions::interrupts::without_interrupts(
            || {
                const MASK: u8 = 0xF0;

                self.disable_nmi();
                let prev = self.read_register(Register::A);
                self.write_register(Register::A, (prev & MASK) | rate);
                self.enable_nmi();
                self.notify_end_of_interrupt();
            }
        );
    }

    /// Enables periodic interrupts.
    pub fn enable_periodic_interrupt(&mut self) { self.enable_interrupt(Interrupt::Periodic); }

    /// Enables alarm interrupts.
    pub fn enable_alarm_interrupt(&mut self) { self.enable_interrupt(Interrupt::Alarm); }

    /// Enables update interrupts.
    pub fn enable_update_interrupt(&mut self) { self.enable_interrupt(Interrupt::Update); }

    /// Enables the specified interrupt.
    fn enable_interrupt(&mut self, interrupt: Interrupt) {
        // OS Dev Wiki: https://wiki.osdev.org/RTC
        instructions::interrupts::without_interrupts(
            || {
                self.disable_nmi();
                let byte = self.read_register(Register::B);
                self.write_register(Register::B, byte | interrupt as u8);
                self.enable_nmi();
                self.notify_end_of_interrupt();
            }
        );
    }

    /// Notifies the end of an interrupt.
    pub fn notify_end_of_interrupt(&mut self) {
        unsafe {
            self.addr.write(Register::C as u8);
            self.data.read();
        }
    }

    /// Reads value from the given register.
    fn read_register(&mut self, reg: Register) -> u8 {
        unsafe {
            self.addr.write(reg as u8);
            self.data.read()
        }
    }

    /// Writes the given value to the specified register.
    fn write_register(&mut self, reg: Register, value: u8) {
        unsafe {
            self.addr.write(reg as u8);
            self.data.write(value);
        }
    }

    /// Returns whether or not an update is in progress.
    fn is_updating(&mut self) -> bool {
        const MASK: u8 = 0x80;

        unsafe {
            self.addr.write(Register::A as u8);
            (self.data.read() & MASK) != 0
        }
    }

    /// Enters a spin loop while an update is in progress.
    fn wait_while_updating(&mut self) {
        while self.is_updating() {
            spin_loop();
        }
    }

    /// Enables Non-Maskable Interrupts (NMI).
    fn enable_nmi(&mut self) {
        const MASK: u8 = 0x7F;

        unsafe {
            let prev = self.addr.read();
            self.addr.write(prev & MASK);
        }
    }

    /// Disables Non-Maskable Interrupts (NMI).
    fn disable_nmi(&mut self) {
        const MASK: u8 = 0x80;

        unsafe {
            let prev = self.addr.read();
            self.addr.write(prev | MASK);
        }
    }
}
