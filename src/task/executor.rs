use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::{ArrayQueue, PopError, PushError};

struct TaskQueue {
    async_task: ArrayQueue<TaskId>,
    timer_task: ArrayQueue<TaskId>,
}

pub enum TaskPriority {
    High,
    Low,
}

impl TaskQueue {
    fn new(array_length: usize) -> Self {
        TaskQueue {
            async_task: ArrayQueue::new(array_length),
            timer_task: ArrayQueue::new(array_length),
        }
    }

    fn push(&self, task_id: TaskId, priority: TaskPriority) -> Result<(), PushError<TaskId>> {
        match priority {
            TaskPriority::High => {
                self.async_task.push(task_id)?;
            }
            TaskPriority::Low => {
                self.timer_task.push(task_id)?;
            }
        }
        Ok(())
    }

    fn pop(&self) -> Result<TaskId, PopError> {
        if let Ok(async_task_id) = self.async_task.pop() {
            return Ok(async_task_id);
        }
        self.timer_task.pop()
    }

    fn is_empty(&self) -> bool {
        self.async_task.is_empty() && self.timer_task.is_empty()
    }
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<TaskQueue>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(TaskQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task, priority: TaskPriority) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id, priority).expect("queue full");
    }

    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();
        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<TaskQueue>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<TaskQueue>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue
            .push(self.task_id, TaskPriority::High)
            .expect("task_queue full");
    }
}

use alloc::task::Wake;

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
