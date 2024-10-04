use core::cmp::min;

use super::TaskRef;
use crate::raw::util::SyncUnsafeCell;

pub(crate) struct TimerQueueItem {
    next: SyncUnsafeCell<Option<TaskRef>>,
}

impl TimerQueueItem {
    pub const fn new() -> Self {
        Self {
            next: SyncUnsafeCell::new(None),
        }
    }
}

pub(crate) struct TimerQueue {
    head: SyncUnsafeCell<Option<TaskRef>>,
}

impl TimerQueue {
    pub const fn new() -> Self {
        Self {
            head: SyncUnsafeCell::new(None),
        }
    }

    //
    pub(crate) unsafe fn update(&self, p: TaskRef) {
        let task = p.header();
        // is alarm set
        if task.expires_at.get() != u64::MAX {
            // if not queued before
            if task.state.timer_enqueue() {
                // insert into timer_queue
                task.timer_queue_item.next.set(self.head.get());
                self.head.set(Some(p));
            }
        }
    }

    pub(crate) unsafe fn next_expiration(&self) -> u64 {
        let mut res = u64::MAX;
        // for every task in timer_queue run this closure
        // which sets the min of all expires
        self.retain(|p| {
            let task = p.header();
            let expires = task.expires_at.get();
            res = min(res, expires);
            expires != u64::MAX
        });
        res
    }

    pub(crate) unsafe fn dequeue_expired(&self, now: u64, on_task: impl Fn(TaskRef)) {
        // for every task in timer_queue run this closure
        self.retain(|p| {
            let task = p.header();
            // alarm expired, on_task = wake_task_no_pend
            // and remove task from timer_queue
            if task.expires_at.get() <= now {
                on_task(p);
                false
            // alarm hasn't expired, step to the nexttask in timer_queue
            } else {
                true
            }
        });
    }

    pub(crate) unsafe fn retain(&self, mut f: impl FnMut(TaskRef) -> bool) {
        let mut prev = &self.head;
        // for every task in timer_queue run 'f'
        // if 'f' gives true, step to the next task in timer_queue
        // if 'f' gives false, remove task from timer_queue and set state accordingly
        while let Some(p) = prev.get() {
            let task = p.header();
            if f(p) {
                // Skip to next
                prev = &task.timer_queue_item.next;
            } else {
                // Remove it
                prev.set(task.timer_queue_item.next.get());
                task.state.timer_dequeue();
            }
        }
    }
}
