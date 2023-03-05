use core::pin::Pin;
use core::task::{Context, Poll};

use conquer_once::spin::OnceCell;
use crossbeam_queue::{ArrayQueue, PopError};
use futures_util::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, layouts, ScancodeSet1};

use crate::{print, println};

/// Capacity of the scancode waiting queue.
const SCANCODE_QUEUE_CAPACITY: usize = 128;
/// A global waiting queue for scancodes.
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
/// A global atomic waker for keyboard interrupts.
static WAKER: AtomicWaker = AtomicWaker::new();

/// Adds the given scancode to the waiting queue.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Ok(_) = queue.push(scancode) {
            WAKER.wake();
        } else {
            println!("WARNING: scancode queue full; dropping keyboard input");
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

/// Scancode Stream
pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    /// Creates a new object.
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(
            || ArrayQueue::new(SCANCODE_QUEUE_CAPACITY)
        ).expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.try_get().expect("uninitialized");

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
pub async fn echo_key_presses() {
    let mut scancodes = ScancodeStream::new();

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
