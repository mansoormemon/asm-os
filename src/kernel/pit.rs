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

use core::arch;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use x86_64::instructions;
use x86_64::instructions::port::Port;

use crate::kernel::cmos::CMOS;
use crate::kernel::idt;
use crate::kernel::idt::IRQ;

// Programmable Interval Timer (PIT | Intel 8253/8254)
//
// The Programmable Interval Timer (PIT) chip (Intel 8253/8254) basically consists of an oscillator,
// a prescaler and 3 independent frequency dividers. Each frequency divider has an output, which is
// used to allow the timer to control external circuitry.
//
// The oscillator used by the PIT chip runs at (roughly) 1.193182 MHz (due to legacy reasons).
//
// The original PC used a single "base oscillator" to generate a frequency of 14.31818 MHz because
// this frequency was commonly used in television circuitry at the time. This base frequency was
// divided by 3 to give a frequency of 4.77272666 MHz that was used by the CPU, and divided by 4 to
// give a frequency of 3.579545 MHz that was used by the CGA video controller. By logically ANDing
// these signals together a frequency equivalent to the base frequency divided by 12 was created.
// This frequency is 1.1931816666 MHz (where the 6666 part is recurring). At the time it was a
// brilliant method of reducing costs, as the 14.31818 MHz oscillator was cheap due to mass production
// and it was cheaper to derive the other frequencies from this than to have several oscillators.
//
// In modern computers, where the cost of electronics is much less, and the CPU and video run at much
// higher frequencies the PIT lives on as a reminder of "the good ole' days".
//
// OS Dev Wiki: https://wiki.osdev.org/Programmable_Interval_Timer

//////////////////
// Calibrations
//////////////////

/// Frequency of the PIT.
pub const FREQUENCY: f64 = 3_579_545.0 / 3.0;

/// Divider for PIT.
const DIVIDER: usize = 1193;

/// Time between successive ticks.
const INTERVAL: f64 = (DIVIDER as f64) / FREQUENCY;

////////////////
// Attributes
////////////////

/// Output channel for the PIT frequency divider.
///
/// Note: Channel 0 is connected directly to IRQ 0, so it is best to use it only for purposes that should
/// generate interrupts. Channel 1 is unusable, and may not even exist. Channel 2 is connected to the
/// PC speaker, but can be used for other purposes without producing audible speaker tones.
///
/// OS Dev Wiki: https://wiki.osdev.org/Programmable_Interval_Timer#Outputs
const OUTPUT_CHANNEL: u8 = 0;

////////////
// States
////////////

/// Flag to check whether PIT is initialized or not.
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Ticks elapsed since PIT was initialized.
static TICKS: AtomicUsize = AtomicUsize::new(0);

/// The latest RTC clock update tick.
static LAST_RTC_UPDATE: AtomicUsize = AtomicUsize::new(0);

//////////////
// Utilities
//////////////

/// Initializes the PIT and sets the relevant interrupt handlers.
pub(crate) fn init() -> Result<(), ()> {
    // The PIT has only 16 bits that are used as frequency divider, which can represent the values from
    // 0 to 65535. Since the frequency can't be divided by 0 in a sane way, many implementations use 0
    // to represent the value 65536.
    let divider = if DIVIDER < 65536 { DIVIDER } else { 0 };

    // Set frequency divider.
    set_pit_frequency_divider(divider as u16, OUTPUT_CHANNEL);

    // Set interrupt handler for timer.
    idt::set_irq_handler(IRQ::Timer, timer_irq_handler);

    // Set interrupt handler for RTC.
    idt::set_irq_handler(IRQ::RTC, rtc_irq_handler);
    // Enable RTC update interrupts.
    CMOS::new().enable_update_interrupt();

    // Update flag.
    IS_INITIALIZED.store(true, Ordering::Relaxed);

    Ok(())
}

/// Returns whether the PIT is initialized or not.
pub(crate) fn is_initialized() -> bool { IS_INITIALIZED.load(Ordering::Relaxed) }

/// Returns the time between two successive ticks.
pub(crate) fn tick_interval() -> f64 { INTERVAL }

/// Returns the ticks elapsed since PIT was initialized.
pub(crate) fn ticks() -> usize { TICKS.load(Ordering::Relaxed) }

/// Returns the latest RTC clock update tick.
pub(crate) fn last_rtc_update() -> usize { LAST_RTC_UPDATE.load(Ordering::Relaxed) }

/// Returns the Read Time-Stamp Counter (RDTSC).
///
/// Reference: https://www.felixcloutier.com/x86/rdtsc
pub(crate) fn rdtsc() -> u64 {
    unsafe {
        arch::x86_64::_mm_lfence();
        arch::x86_64::_rdtsc()
    }
}

/// Returns the time elapsed since the PIT was initialized.
pub(crate) fn uptime() -> f64 { (ticks() as f64) * tick_interval() }

/// Halts the CPU.
///
/// Note: It restores the state of interrupts (whether enabled or disabled) after execution.
pub(crate) fn halt() {
    let disabled = !instructions::interrupts::are_enabled();
    instructions::interrupts::enable_and_hlt();
    if disabled { instructions::interrupts::disable(); }
}

/// Halts the CPU for the specified duration.
pub(crate) fn sleep(seconds: f64) {
    let start = uptime();
    while uptime() - start < seconds {
        halt();
    }
}

/// Sets the frequency divider for the PIT.
pub(crate) fn set_pit_frequency_divider(divider: u16, channel: u8) {
    instructions::interrupts::without_interrupts(
        || {
            const TOTAL_CHANNELS: usize = 3;

            const DATA_PORTS: [u16; TOTAL_CHANNELS] = [0x40u16, 0x41u16, 0x42u16];
            const CMD_PORT: u16 = 0x43;

            const OP_MODE: u16 = 0x6;
            const ACCESS_MODE: u16 = 0x30;
            const CHANNEL_BIT: u8 = 6;

            let channel_mask: u16 = (channel << CHANNEL_BIT) as u16;

            let bytes = divider.to_le_bytes();
            let mut cmd = Port::new(CMD_PORT);
            let mut data = Port::new(DATA_PORTS[channel as usize]);
            unsafe {
                cmd.write(channel_mask | ACCESS_MODE | OP_MODE);
                for byte in bytes {
                    data.write(byte);
                }
            }
        }
    )
}

//////////////
// Handlers
//////////////

/// Interrupt handler for timer.
fn timer_irq_handler() { TICKS.fetch_add(1, Ordering::Relaxed); }

/// Interrupt handler for RTC.
fn rtc_irq_handler() {
    LAST_RTC_UPDATE.store(ticks(), Ordering::Relaxed);
    CMOS::new().notify_end_of_interrupt();
}
