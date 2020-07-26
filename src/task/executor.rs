use super::{Task, TaskId};
use crate::common::Once;
use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
    task::Wake,
};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

struct TaskWaker {
    task_id: TaskId,
    wake_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn wake_task(&self) {
        self.wake_queue
            .push(self.task_id)
            .expect("wake_queue is full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

/// Clone of `waiting_for_task_queue` in the Executor.
static WFTQ: Once<Arc<ArrayQueue<Task>>> = Once::new();

/// Queues the task to be run in the next poll.
/// Will fail if the queue is full, or the executor has not been initialized.
pub fn spawn_task(task: Task) {
    use log::warn;

    if let Some(s) = WFTQ.get() {
        if s.push(task).is_err() {
            warn!("can't spawn task, queue full!");
        }
    } else {
        warn!("executor not initialized, can't spawn task");
    }
}

/// How many tasks / wakers can be queued.
pub const TASK_QUEUE_LIMIT: usize = 100;

/// Simple FIFO task executor. Supports wakers.
pub struct Executor {
    task_queue: VecDeque<Task>,
    waiting_for_task_queue: Arc<ArrayQueue<Task>>,
    waiting_tasks: BTreeMap<TaskId, Task>,
    wake_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        let waiting_for_task_queue = Arc::new(ArrayQueue::new(TASK_QUEUE_LIMIT));
        WFTQ.try_init(waiting_for_task_queue.clone());
        Executor {
            task_queue: VecDeque::new(),
            waiting_for_task_queue,
            waiting_tasks: BTreeMap::new(),
            wake_queue: Arc::new(ArrayQueue::new(TASK_QUEUE_LIMIT)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.wake_tasks();
            self.run_ready_tasks();
            #[cfg(target_arch = "x86_64")]
            self.sleep_if_idle(); // Getting here means that there are no tasks left in `task_queue`
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_interrupts_and_hlt};

        // Return early, no need to disable interrupts
        if !self.wake_queue.is_empty() {
            return;
        }

        interrupts::disable();
        // If an interrupt happened inbetween, interrupts will be enabled
        if self.wake_queue.is_empty() {
            enable_interrupts_and_hlt();
        } else {
            interrupts::enable();
        }
    }

    fn create_waker(&self, task_id: TaskId) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            wake_queue: self.wake_queue.clone(),
        }))
    }

    fn run_ready_tasks(&mut self) {
        while let Ok(task) = self.waiting_for_task_queue.pop() {
            self.task_queue.push_back(task);
        }
        while let Some(mut task) = self.task_queue.pop_front() {
            let task_id = task.id;
            // Create a new `Waker` if it isn't already in the cache.
            #[allow(clippy::map_entry)]
            if !self.waker_cache.contains_key(&task_id) {
                self.waker_cache.insert(task_id, self.create_waker(task_id));
            }

            let waker = self
                .waker_cache
                .get(&task_id)
                .expect("There should be a waker with this key. I hope.");
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // Task is done, remove cached waker
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {
                    // Task isn't done, back to waiting list
                    if self.waiting_tasks.insert(task_id, task).is_some() {
                        panic!("Task with same ID already in waiting_tasks! Literally how");
                    }
                }
            }
        }
    }

    fn wake_tasks(&mut self) {
        while let Ok(task_id) = self.wake_queue.pop() {
            if let Some(task) = self.waiting_tasks.remove(&task_id) {
                self.task_queue.push_back(task);
            }
        }
    }
}
