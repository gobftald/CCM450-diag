//! Timer waiter queue.
//!
//! This module implements the timer queue, which is managed by the time driver.
//! The timer queue contains wakers and their expiration times, and is used to
//! wake tasks at the correct time.

use core::cell::Cell;

// 17
struct TimerQueueInner {
    queue: RawQueue,
    //alarm: Option<AlarmHandle>,
}

// 22
pub(crate) struct TimerQueue {
    inner: Cell<TimerQueueInner>,
    //priority: Priority,
    #[cfg(not(single_queue))]
    context: Cell<*mut ()>,
}

impl TimerQueue {
    //pub(crate) const fn new(prio: Priority) -> Self
    pub(crate) const fn new() -> Self {
        Self {
            //inner: Mutex::const_new(
            //    RawPriorityLimitedMutex::new(prio),
            //    RefCell::new(TimerQueueInner {
            //        queue: RawQueue::new(),
            //        alarm: None,
            //    }),
            //),
            inner: Cell::new(TimerQueueInner {
                queue: RawQueue::new(),
                //alarm: None,
            }),
            //priority: prio,
            #[cfg(not(single_queue))]
            context: Cell::new(core::ptr::null_mut()),
        }
    }
}
