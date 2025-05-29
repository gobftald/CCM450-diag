//! Timer waiter queue.
//!
//! This module implements the timer queue, which is managed by the time driver.
//! The timer queue contains wakers and their expiration times, and is used to
//! wake tasks at the correct time.

use core::cell::Cell;

// 12
use esp_hal::interrupt::Priority;
use queue_impl::RawQueue;

// 15
use crate::time_driver::{set_up_alarm, AlarmHandle};

// 17
struct TimerQueueInner {
    queue: RawQueue,
    alarm: Option<AlarmHandle>,
}

// 22
pub(crate) struct TimerQueue {
    //inner: Mutex<RawPriorityLimitedMutex, RefCell<TimerQueueInner>>,
    inner: Cell<TimerQueueInner>,
    priority: Priority,
    #[cfg(not(single_queue))]
    context: Cell<*mut ()>,
}

// 29
//unsafe impl Sync for TimerQueue {}

// 31
impl TimerQueue {
    // 32
    pub(crate) const fn new(prio: Priority) -> Self {
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
                alarm: None,
            }),
            priority: prio,
            #[cfg(not(single_queue))]
            context: Cell::new(core::ptr::null_mut()),
        }
    }

    // 62
    pub fn dispatch(&mut self) {
        let now = esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros();
        self.arm_alarm(now);
    }

    // 69
    fn arm_alarm(&mut self, mut next_expiration: u64) {
        loop {
            let set = /*self.inner.lock(|inner|*/ {
            //let mut q = inner.borrow_mut();
            let q = self.inner.get_mut();
            next_expiration = q.queue.next_expiration(next_expiration);

            let alarm = q
                .alarm
                //.get_or_insert_with(|| set_up_alarm(self.priority, self.context()));
                .get_or_insert_with(|| set_up_alarm(self.priority));
            alarm.update(next_expiration)
            //});
            };
            if set {
                break;
            }
        }
    }
}

#[cfg(integrated_timers)]
// 97
mod queue_impl {
    use core::{cell::Cell, cmp::min};

    use embassy_executor::raw::TaskRef;

    /// Copy of the embassy integrated timer queue, that clears the owner upon
    /// dequeueing.
    // 105
    pub(super) struct RawQueue {
        head: Cell<Option<TaskRef>>,
    }

    // 109
    impl RawQueue {
        /// Creates a new timer queue.
        // 111
        pub const fn new() -> Self {
            Self {
                head: Cell::new(None),
            }
        }

        /// Dequeues expired timers and returns the next alarm time.
        ///
        /// The provided callback will be called for each expired task. Tasks
        /// that never expire will be removed, but the callback will not
        /// be called.
        // 149
        pub fn next_expiration(&mut self, now: u64) -> u64 {
            let mut next_expiration = u64::MAX;

            self.retain(|p| {
                let item = p.timer_queue_item();
                let expires = item.expires_at.get();

                if expires <= now {
                    // Timer expired, process task.
                    embassy_executor::raw::wake_task(p);
                    false
                } else {
                    // Timer didn't yet expire, or never expires.
                    next_expiration = min(next_expiration, expires);
                    expires != u64::MAX
                }
            });

            next_expiration
        }

        // 170
        fn retain(&self, mut f: impl FnMut(TaskRef) -> bool) {
            let mut prev = &self.head;
            while let Some(p) = prev.get() {
                if unsafe { p == TaskRef::dangling() } {
                    // prev was the last item, stop
                    break;
                }
                let item = p.timer_queue_item();
                if f(p) {
                    // Skip to next
                    prev = &item.next;
                } else {
                    // Remove it
                    prev.set(item.next.get());
                    /*
                     * commenting this out eliminates the only difference
                     * betwwen this and the copied/original implementation
                    // Clear owner
                    unsafe {
                        // SAFETY: our payload is an AtomicPtr.
                        item.payload
                            .as_ref::<AtomicPtr<()>>()
                            .store(ptr::null_mut(), Ordering::Relaxed);
                    }
                    */
                    item.next.set(None);
                }
            }
        }
    }
}
