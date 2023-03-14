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

use crate::kernel;

///////////////
// Utilities
///////////////

/// Returns where the PIT is initialized or not.
pub fn is_timer_initialized() -> bool {
    kernel::pit::is_initialized()
}

/// Returns the duration between successive ticks.
pub fn tick_interval() -> f64 {
    kernel::pit::tick_interval()
}

/// Returns the ticks elapsed since PIT was initialized.
pub fn ticks() -> usize {
    kernel::pit::ticks()
}

/// Returns the latest RTC clock update tick.
pub fn last_rtc_update() -> usize {
    kernel::pit::last_rtc_update()
}

/// Returns the Read Time-Stamp Counter (RDTSC).
///
/// Reference: https://www.felixcloutier.com/x86/rdtsc
pub fn rdtsc() -> u64 {
    kernel::pit::rdtsc()
}

/// Returns the time elapsed since the PIT was initialized.
pub fn uptime() -> f64 {
    kernel::pit::uptime()
}

/// Halts the CPU.
///
/// Note: It restores the state of interrupts (whether enabled or disabled) after execution.
pub fn halt() {
    kernel::pit::halt();
}

/// Halts the CPU for the specified duration.
pub fn sleep(seconds: f64) {
    kernel::pit::sleep(seconds);
}

/// Powers off the machine.
pub fn power_off() {
    kernel::acpi::power_off();
}

/// Returns the instantaneous UNIX timestamp.
pub fn timestamp() -> u64 {
    use crate::api::chrono::{DAYS_IN_LEAP_YEAR, DAYS_IN_YEAR, SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE};

    const EPOCH: u64 = 1970;
    const DAYS_BEFORE_MONTH: [u64; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

    let is_leap_year = |year: u64| -> bool {
        (year % 400 == 0) || ((year % 4 == 0) && (year % 100 != 0))
    };

    let days_before_year = |year: u64| -> u64 {
        (EPOCH..year).fold(0, |days: u64, y: u64| -> u64 {
            days + if is_leap_year(y) { DAYS_IN_LEAP_YEAR } else { DAYS_IN_YEAR }
        })
    };

    let days_before_month = |year: u64, month: u64| -> u64 {
        let leap_day = is_leap_year(year) && month > 2;
        DAYS_BEFORE_MONTH[(month as usize) - 1] + (leap_day as u64)
    };

    let rtc = kernel::cmos::RTC::new();
    let timestamp = SECONDS_IN_DAY * days_before_year(rtc.year as u64)
        + SECONDS_IN_DAY * days_before_month(rtc.year as u64, rtc.month as u64)
        + SECONDS_IN_DAY * ((rtc.day - 1) as u64)
        + SECONDS_IN_HOUR * rtc.hour as u64
        + SECONDS_IN_MINUTE * rtc.minute as u64
        + rtc.second as u64;
    timestamp
}
