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

use core::fmt;

use crate::drv::clk;

///////////////
// Globals
///////////////

pub const SECONDS_IN_MINUTE: u64 = 60;
pub const SECONDS_IN_HOUR: u64 = 3600;
pub const SECONDS_IN_DAY: u64 = 86400;
pub const MINUTES_IN_HOUR: u64 = 60;
pub const MINUTES_IN_DAY: u64 = 1440;
pub const HOURS_IN_DAY: u64 = 24;
pub const HOURS_IN_WEEK: u64 = 168;
pub const DAYS_IN_WEEK: u64 = 7;
pub const DAYS_IN_YEAR: u64 = 365;
pub const DAYS_IN_LEAP_YEAR: u64 = 366;
pub const MONTHS_IN_YEAR: u64 = 12;

pub const WEEKDAYS: [Weekday; DAYS_IN_WEEK as usize] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

pub const MONTHS: [Month; MONTHS_IN_YEAR as usize] = [
    Month::January,
    Month::February,
    Month::March,
    Month::April,
    Month::May,
    Month::June,
    Month::July,
    Month::August,
    Month::September,
    Month::October,
    Month::November,
    Month::December,
];

///////////////
/// Weekday
///////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Weekday {
    Monday = 0x0,
    Tuesday = 0x1,
    Wednesday = 0x2,
    Thursday = 0x3,
    Friday = 0x4,
    Saturday = 0x5,
    Sunday = 0x6,
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Weekday::Monday => write!(f, "Monday"),
            Weekday::Tuesday => write!(f, "Tuesday"),
            Weekday::Wednesday => write!(f, "Wednesday"),
            Weekday::Thursday => write!(f, "Thursday"),
            Weekday::Friday => write!(f, "Friday"),
            Weekday::Saturday => write!(f, "Saturday"),
            Weekday::Sunday => write!(f, "Sunday"),
        }
    }
}

/////////////
/// Month
/////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Month {
    January = 0x0,
    February = 0x1,
    March = 0x2,
    April = 0x3,
    May = 0x4,
    June = 0x5,
    July = 0x6,
    August = 0x7,
    September = 0x8,
    October = 0x9,
    November = 0xA,
    December = 0xB,
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Month::January => write!(f, "January"),
            Month::February => write!(f, "February"),
            Month::March => write!(f, "March"),
            Month::April => write!(f, "April"),
            Month::May => write!(f, "May"),
            Month::June => write!(f, "June"),
            Month::July => write!(f, "July"),
            Month::August => write!(f, "August"),
            Month::September => write!(f, "September"),
            Month::October => write!(f, "October"),
            Month::November => write!(f, "November"),
            Month::December => write!(f, "December"),
        }
    }
}

//////////////////
/// Time Point
//////////////////
#[derive(PartialEq, Eq)]
pub struct TimePoint {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        )
    }
}

/////////////
/// Clock
/////////////
pub struct Clock;

impl Clock {
    /// Returns the current time.
    pub fn now() -> TimePoint {
        let rtc = clk::cmos::RTC::new();

        TimePoint {
            year: rtc.year,
            month: rtc.month,
            day: rtc.day,
            hour: rtc.hour,
            minute: rtc.minute,
            second: rtc.second,
        }
    }
}
