use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll};

pub mod executor;

/// Task ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TaskID(u64);

impl TaskID {
    /// Creates a new object.
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskID(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Task.
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
