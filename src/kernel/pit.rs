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

use core::arch;
use core::sync::atomic::{AtomicUsize, Ordering};

use x86_64::instructions;
use x86_64::instructions::port::Port;

use crate::kernel::cmos::CMOS;
use crate::kernel::interrupts::{self, InterruptIndex};

/// Frequency of the PIT.
pub const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0;

/// Divider for PIT.
const PIT_DIVIDER: usize = 1193;

/// Time between successive ticks.
const PIT_INTERVAL: f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

/// Output channel for the PIT frequency divider.
///
/// Note: Channel 0 is connected directly to IRQ 0, so it is best to use it only for purposes that should
/// generate interrupts. Channel 1 is unusable, and may not even exist. Channel 2 is connected to the
/// PC speaker, but can be used for other purposes without producing audible speaker tones.
///
/// OS Dev Wiki: https://wiki.osdev.org/Programmable_Interval_Timer#Outputs
const OUTPUT_CHANNEL: u8 = 0;

/// Ticks elapsed since PIT was initialized.
static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);

/// The latest RTC clock update tick.
static LAST_RTC_UPDATE: AtomicUsize = AtomicUsize::new(0);

/// Returns the time between successive ticks.
pub fn time_between_ticks() -> f64 {
    PIT_INTERVAL
}

/// Returns the ticks elapsed since PIT was initialized.
pub fn ticks() -> usize {
    PIT_TICKS.load(Ordering::Relaxed)
}

/// Returns the latest RTC clock update tick.
pub fn last_rtc_update() -> usize {
    LAST_RTC_UPDATE.load(Ordering::Relaxed)
}

/// Read Time-Stamp Counter (RDTSC).
///
/// Reference: https://www.felixcloutier.com/x86/rdtsc
pub fn rdtsc() -> u64 {
    unsafe {
        arch::x86_64::_mm_lfence();
        arch::x86_64::_rdtsc()
    }
}

/// Time elapsed since the PIT was initialized.
pub fn uptime() -> f64 {
    (ticks() as f64) * time_between_ticks()
}

/// Halts the CPU.
///
/// Note: It restores the state of interrupts (whether enabled or disabled) after execution.
pub fn halt() {
    let disabled = !instructions::interrupts::are_enabled();
    instructions::interrupts::enable_and_hlt();
    if disabled {
        instructions::interrupts::disable();
    }
}

/// Halts the CPU for the specified duration.
pub fn sleep(seconds: f64) {
    let start = uptime();
    while uptime() - start < seconds {
        halt();
    }
}

/// Sets the frequency divider of the PIT.
pub fn set_pit_frequency_divider(divider: u16, channel: u8) {
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

/// Interrupt handler for timer.
fn timer_interrupt_handler() {
    PIT_TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Interrupt handler for RTC.
fn rtc_interrupt_handler() {
    LAST_RTC_UPDATE.store(ticks(), Ordering::Relaxed);
    CMOS::new().notify_end_of_interrupt();
}

/// Initializes the PIT and sets the relevant interrupt handlers.
pub fn init() {
    // The PIT has only 16 bits that are used as frequency divider, which can represent the values from
    // 0 to 65535. Since the frequency can't be divided by 0 in a sane way, many implementations use 0
    // to represent the value 65536.
    let divider = if PIT_DIVIDER < 65536 { PIT_DIVIDER } else { 0 };

    set_pit_frequency_divider(divider as u16, OUTPUT_CHANNEL);

    interrupts::set_interrupt_handler(InterruptIndex::Timer, timer_interrupt_handler);

    interrupts::set_interrupt_handler(InterruptIndex::RTC, rtc_interrupt_handler);
    CMOS::new().enable_update_interrupt();
}
