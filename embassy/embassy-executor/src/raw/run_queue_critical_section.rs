use core::cell::Cell;

use super::TaskRef;

// 7
pub(crate) struct RunQueueItem {
    //next: Mutex<Cell<Option<TaskRef>>>,
    next: Cell<Option<TaskRef>>,
}

// 11
impl RunQueueItem {
    pub const fn new() -> Self {
        Self {
            //next: Mutex::new(Cell::new(None)),
            next: Cell::new(None),
        }
    }
}

// 30
pub(crate) struct RunQueue {
    //head: Mutex<Cell<Option<TaskRef>>>,
    head: Cell<Option<TaskRef>>,
}

// 34
impl RunQueue {
    // 35
    pub const fn new() -> Self {
        Self {
            //head: Mutex::new(Cell::new(None)),
            head: Cell::new(None),
        }
    }

    /// Enqueues an item. Returns true if the queue was empty.
    ///
    /// # Safety
    ///
    /// `item` must NOT be already enqueued in any queue.
    #[inline(always)]
    // 47
    pub(crate) unsafe fn enqueue(&self, task: TaskRef) -> bool {
        //let prev = self.head.borrow(cs).replace(Some(task));
        let prev = self.head.replace(Some(task));
        //task.header().run_queue_item.next.borrow(cs).set(prev);
        task.header().run_queue_item.next.set(prev);

        prev.is_none()
    }
}
