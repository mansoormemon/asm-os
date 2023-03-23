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

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::{Context, Poll, Waker};

use crossbeam_queue::ArrayQueue;
use x86_64::instructions;

use crate::kernel::task::{Task, TaskID};

////////////////
// Attributes
////////////////

/// Size of waiting queue for tasks.
pub const QUEUE_SIZE: usize = 128;

////////////////
/// Executor
////////////////
pub struct Executor {
    tasks: BTreeMap<TaskID, Task>,
    task_queue: Arc<ArrayQueue<TaskID>>,
    waker_cache: BTreeMap<TaskID, Waker>,
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
        if let Some(_) = self.tasks.insert(task_id, task) { panic!("a task with the same ID already exists"); }
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
            let waker = waker_cache.entry(task_id).or_insert_with(
                || { WakerWrapper::new(task_id, task_queue.clone()) }
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

/////////////////////
/// Waker Wrapper
/////////////////////
struct WakerWrapper {
    task_id: TaskID,
    task_queue: Arc<ArrayQueue<TaskID>>,
}

impl WakerWrapper {
    /// Creates a new `Waker`.
    fn new(task_id: TaskID, task_queue: Arc<ArrayQueue<TaskID>>) -> Waker {
        Waker::from(Arc::new(WakerWrapper {
            task_id,
            task_queue,
        }))
    }

    /// Pushes the task back to the waiting queue when it's ready for execution.
    fn wake_task(&self) { self.task_queue.push(self.task_id).expect("task queue is full"); }
}

impl Wake for WakerWrapper {
    fn wake(self: Arc<Self>) { self.wake_task(); }

    fn wake_by_ref(self: &Arc<Self>) { self.wake_task(); }
}
