use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::{Context, Poll, Waker};

use crossbeam_queue::ArrayQueue;
use x86_64::instructions;

use crate::kernel::task::{Task, TaskId};

/// Size of waiting queue for tasks.
pub const QUEUE_SIZE: usize = 128;

/// Executor
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    /// Creates a new object.
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(QUEUE_SIZE)),
            waker_cache: BTreeMap::new(),
        }
    }

    /// Spawns the given task.
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if let Some(_) = self.tasks.insert(task_id, task) {
            panic!("a task with the same ID already exists");
        }
        self.task_queue.push(task_id).expect("task queue is full");
    }

    /// Runs all the ready tasks, halts the CPU otherwise.
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    /// Runs all the ready tasks.
    fn run_ready_tasks(&mut self) {
        let Self { tasks, task_queue, waker_cache } = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(
                    || WakerWrapper::new(task_id, task_queue.clone())
                );
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    /// Halts the CPU if there are no tasks.
    fn sleep_if_idle(&self) {
        instructions::interrupts::disable();
        if self.task_queue.is_empty() {
            instructions::interrupts::enable_and_hlt();
        } else {
            instructions::interrupts::enable();
        }
    }
}

/// Waker Wrapper
struct WakerWrapper {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl WakerWrapper {
    /// Creates a new Waker.
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(WakerWrapper {
            task_id,
            task_queue,
        }))
    }

    /// Pushes the task back to the waiting queue when it's ready for execution.
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task queue is full");
    }
}

impl Wake for WakerWrapper {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
