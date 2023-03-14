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

use core::pin::Pin;
use core::task::{Context, Poll};

use conquer_once::spin::OnceCell;
use crossbeam_queue::{ArrayQueue, PopError};
use futures_util::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, layouts, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;

use crate::{print, success, warning};
use crate::kernel::interrupts::{self, InterruptIndex};

/// Capacity of the scancode waiting queue.
const SCANCODE_QUEUE_CAPACITY: usize = 128;
/// A global waiting queue for scancodes.
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
/// A global atomic waker for keyboard interrupts.
static WAKER: AtomicWaker = AtomicWaker::new();

/// Adds the given scancode to the waiting queue.
fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Ok(_) = queue.push(scancode) {
            WAKER.wake();
        } else {
            warning!("scancode queue full; dropping keyboard input");
        }
    } else {
        warning!("scancode queue uninitialized");
    }
}

lazy_static! {
    /// A global interface for scancode stream.
    pub static ref READER: Mutex<ScancodeStream> = Mutex::new(ScancodeStream::new());
}

/// Scancode Stream.
pub struct ScancodeStream {
    __unused: (),
}

impl ScancodeStream {
    /// Creates a new object.
    pub fn new() -> Self {
        ScancodeStream { __unused: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.try_get().expect("scancode queue uninitialized");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(PopError) => {
                Poll::Pending
            }
        }
    }
}

/// Echoes the scancodes on key-press.
pub async fn echo() {
    let mut scancodes = READER.lock();

    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(ch) => print!("{}", ch),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}

/// An interrupt handler for keyboard interrupts.
fn keyboard_interrupt_handler() {
    const PORT: u16 = 0x60;

    let mut port = Port::new(PORT);
    let scancode: u8 = unsafe { port.read() };

    // Add the scancode to the waiting queue.
    add_scancode(scancode);
}

pub(crate) fn init() {
    SCANCODE_QUEUE.try_init_once(
        || ArrayQueue::new(SCANCODE_QUEUE_CAPACITY)
    ).expect("scancode queue should only be initialized once");
    success!("Scancode queue initialized");

    interrupts::set_interrupt_handler(InterruptIndex::Keyboard, keyboard_interrupt_handler);
    success!("Keyboard initialized");
}
