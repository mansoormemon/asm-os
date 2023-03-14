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

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll};

pub use executor::Executor;

mod executor;

////////////////
// Attributes
////////////////

/// Keeps track of IDs.
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

///////////////
/// Task ID
///////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TaskID(u64);

impl TaskID {
    /// Creates a new object.
    fn new() -> Self {
        TaskID(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

////////////
/// Task
////////////
pub struct Task {
    id: TaskID,
    future: Pin<Box<dyn Future<Output=()>>>,
}

impl Task {
    /// Creates a new object.
    pub fn new(future: impl Future<Output=()> + 'static) -> Self {
        Task {
            id: TaskID::new(),
            future: Box::pin(future),
        }
    }

    /// Polls the inner future using the given context.
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
