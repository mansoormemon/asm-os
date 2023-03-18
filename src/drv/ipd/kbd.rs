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

use core::sync::atomic::{AtomicBool, Ordering};

use pc_keyboard::{DecodedKey, Error, HandleControl, Keyboard, KeyCode, KeyEvent, KeyState, ScancodeSet1};
use pc_keyboard::layouts::{Azerty, Dvorak104Key, Us104Key};
use spin::Mutex;
use x86_64::instructions::port::Port;

use crate::api;
use crate::api::kbd::Layout;
use crate::cenc::ascii;
use crate::dev::console;
use crate::success;
use crate::x86::kernel::idt;
use crate::x86::kernel::idt::IRQ;

/////////////
// Mutexes
/////////////

/// A keyboard interface with mutex protection.
static KEYBOARD: Mutex<Option<LayoutWrapper>> = Mutex::new(None);

////////////
// States
////////////

/// State of the ALT key.
pub static ALT: AtomicBool = AtomicBool::new(false);
/// State of the CTRL key.
pub static CTRL: AtomicBool = AtomicBool::new(false);
/// State of the SHIFT key.
pub static SHIFT: AtomicBool = AtomicBool::new(false);

//////////////////////
/// Layout Wrapper
//////////////////////
enum LayoutWrapper {
    AZERTY(Keyboard<Azerty, ScancodeSet1>),
    Dvorak(Keyboard<Dvorak104Key, ScancodeSet1>),
    QWERTY(Keyboard<Us104Key, ScancodeSet1>),
}

impl LayoutWrapper {
    /// Creates an object from layout.
    fn from(lyt: Layout) -> Self {
        match lyt {
            Layout::AZERTY => {
                LayoutWrapper::AZERTY(Keyboard::new(ScancodeSet1::new(), Azerty, HandleControl::MapLettersToUnicode))
            }
            Layout::Dvorak => {
                LayoutWrapper::Dvorak(Keyboard::new(ScancodeSet1::new(), Dvorak104Key, HandleControl::MapLettersToUnicode))
            }
            Layout::QWERTY => {
                LayoutWrapper::QWERTY(Keyboard::new(ScancodeSet1::new(), Us104Key, HandleControl::MapLettersToUnicode))
            }
        }
    }

    /// Unwraps the object and returns the corresponding layout.
    fn unwrap(&self) -> Layout {
        match self {
            LayoutWrapper::AZERTY(_) => Layout::AZERTY,
            LayoutWrapper::Dvorak(_) => Layout::Dvorak,
            LayoutWrapper::QWERTY(_) => Layout::QWERTY,
        }
    }

    /// Processes a byte inputted from the keyboard.
    fn add_byte(&mut self, scancode: u8) -> Result<Option<KeyEvent>, Error> {
        match self {
            LayoutWrapper::AZERTY(keyboard) => keyboard.add_byte(scancode),
            LayoutWrapper::Dvorak(keyboard) => keyboard.add_byte(scancode),
            LayoutWrapper::QWERTY(keyboard) => keyboard.add_byte(scancode),
        }
    }

    /// Processes a key event and returns a decoded key.
    fn process_keyevent(&mut self, event: KeyEvent) -> Option<DecodedKey> {
        match self {
            LayoutWrapper::AZERTY(keyboard) => keyboard.process_keyevent(event),
            LayoutWrapper::Dvorak(keyboard) => keyboard.process_keyevent(event),
            LayoutWrapper::QWERTY(keyboard) => keyboard.process_keyevent(event),
        }
    }
}

///////////////
// Utilities
///////////////

/// Initializes the keyboard.
pub(crate) fn init(lyt: Layout) {
    set_layout(lyt);
    idt::set_irq_handler(IRQ::Keyboard, keyboard_irq_handler);
    success!("Keyboard initialized");
}

/// Returns the layout.
pub(crate) fn get_layout() -> Layout {
    let mut mutex_guarded_kbd = KEYBOARD.lock();
    let ref mut keyboard = mutex_guarded_kbd.as_mut().expect("keyboard layout not set");
    keyboard.unwrap()
}

/// Sets the layout.
pub(crate) fn set_layout(lyt: Layout) {
    let mut keyboard = KEYBOARD.lock();
    keyboard.replace(LayoutWrapper::from(lyt));
}

/// Resets the layout.
pub(crate) fn reset_layout() { set_layout(api::kbd::Default::LAYOUT); }

/// Returns a byte read from the input port.
fn read_scancode() -> u8 {
    const PORT: u16 = 0x60;

    let mut port = Port::new(PORT);
    unsafe { port.read() }
}

/// Sends the pressed key to the console.
fn send_key(c: char) { console::key_handle(c); }

/// Sends a Control Sequence Introducer (CSI) to the console.
fn send_csi(code: &'static str) {
    send_key('\x1B');
    send_key('[');
    for byte in code.bytes() {
        send_key(byte as char);
    }
}

//////////////
// Handlers
//////////////

/// An irq handler for keyboard.
fn keyboard_irq_handler() {
    const H_TAB_KEY: char = ascii::HT as char;
    const DEL_KEY: char = ascii::DEL as char;

    let mut mutex_guarded_kbd = KEYBOARD.lock();
    let keyboard = mutex_guarded_kbd.as_mut().unwrap();

    let scancode: u8 = read_scancode();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        match key_event.code {
            KeyCode::LAlt | KeyCode::RAltGr => {
                ALT.store(key_event.state == KeyState::Down, Ordering::Relaxed)
            }
            KeyCode::LShift | KeyCode::RShift => {
                SHIFT.store(key_event.state == KeyState::Down, Ordering::Relaxed)
            }
            KeyCode::LControl | KeyCode::RControl => {
                CTRL.store(key_event.state == KeyState::Down, Ordering::Relaxed)
            }
            _ => {}
        }

        let is_alt = ALT.load(Ordering::Relaxed);
        let is_ctrl = CTRL.load(Ordering::Relaxed);
        let is_shift = SHIFT.load(Ordering::Relaxed);

        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::RawKey(KeyCode::ArrowUp) => send_csi("A"),
                DecodedKey::RawKey(KeyCode::ArrowDown) => send_csi("B"),
                DecodedKey::RawKey(KeyCode::ArrowRight) => send_csi("C"),
                DecodedKey::RawKey(KeyCode::ArrowLeft) => send_csi("D"),
                DecodedKey::Unicode(H_TAB_KEY) if is_shift => send_csi("Z"),
                DecodedKey::Unicode(DEL_KEY) if is_alt && is_ctrl => api::system::reboot(),
                DecodedKey::Unicode(key) => send_key(key),
                _ => {}
            }
        }
    }
}
