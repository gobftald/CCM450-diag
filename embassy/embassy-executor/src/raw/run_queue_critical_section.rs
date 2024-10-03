// for defmt logs (brings defmt runtime into scope)
use esp_println as _;

use core::cell::Cell;

use critical_section::{CriticalSection, Mutex};

use super::TaskRef;

pub(crate) struct RunQueueItem {
    next: Mutex<Cell<Option<TaskRef>>>,
}

impl RunQueueItem {
    pub const fn new() -> Self {
        Self {
            next: Mutex::new(Cell::new(None)),
        }
    }
}

pub(crate) struct RunQueue {
    head: Mutex<Cell<Option<TaskRef>>>,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            head: Mutex::new(Cell::new(None)),
        }
    }

    /// Enqueues an item. Returns true if the queue was empty.
    #[inline(always)]
    pub(crate) unsafe fn enqueue(&self, task: TaskRef) -> bool {
        critical_section::with(|cs| {
            let prev = self.head.borrow(cs).replace(Some(task));
            task.header().run_queue_item.next.borrow(cs).set(prev);

            prev.is_none()
        })
    }

    /// Empty the queue, then call `on_task` for each task that was in the queue.
    /// NOTE: It is OK for `on_task` to enqueue more tasks. In this case they're left in the queue
    /// and will be processed by the *next* call to `dequeue_all`, *not* the current one.
    pub(crate) fn dequeue_all(&self, on_task: impl Fn(TaskRef)) {
        // Atomically empty the queue.
        let mut next = critical_section::with(|cs| self.head.borrow(cs).take());

        // Iterate the linked list of tasks that were previously in the queue.
        while let Some(task) = next {
            // If the task re-enqueues itself, the `next` pointer will get overwritten.
            // Therefore, first read the next pointer, and only then process the task.

            // safety: we know if the task is enqueued, no one else will touch the `next` pointer.
            let cs = unsafe { CriticalSection::new() };
            next = task.header().run_queue_item.next.borrow(cs).get();

            on_task(task);
        }
    }
}
