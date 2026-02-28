//! Semaphores and mutexes.
//!
//! This module provides the [`Semaphore`] type, which implements counting semaphores and mutexes.

// 5
use core::ptr::NonNull;

// 7
use esp_hal::{system::Cpu, time::Instant};
use esp_sync::NonReentrantMutex;

// 10
use crate::{
    SCHEDULER,
    run_queue::Priority,
    task::{TaskExt, TaskPtr},
    wait_queue::WaitQueue,
};

// 17
enum SemaphoreInner {
    Counting {
        current: u32,
        max: u32,
        waiting: WaitQueue,
    },
    Mutex {
        recursive: bool,
        owner: Option<TaskPtr>,
        original_priority: Priority,
        lock_counter: u32,
        waiting: WaitQueue,
    },
}

// 32
impl SemaphoreInner {
    fn try_take(&mut self) -> bool {
        match self {
            SemaphoreInner::Counting { current, .. } => {
                if *current > 0 {
                    *current -= 1;
                    true
                } else {
                    false
                }
            }
            SemaphoreInner::Mutex {
                recursive,
                owner,
                lock_counter,
                original_priority,
                ..
            } => {
                SCHEDULER.with(|scheduler| {
                    let current = scheduler.current_task(Cpu::current());
                    if let Some(owner) = owner {
                        if *owner == current && *recursive {
                            *lock_counter += 1;
                            true
                        } else {
                            // We can't lock the mutex. Make sure the mutex holder has a high enough
                            // priority to avoid priority inversion.
                            let current_priority = current.priority(&mut scheduler.run_queue);
                            if owner.priority(&mut scheduler.run_queue) < current_priority {
                                owner.set_priority(&mut scheduler.run_queue, current_priority);
                                scheduler.resume_task(*owner);
                            }
                            false
                        }
                    } else {
                        *owner = Some(current);
                        *lock_counter += 1;
                        *original_priority = current.priority(&mut scheduler.run_queue);
                        true
                    }
                })
            }
        }
    }

    // 77
    fn try_take_from_isr(&mut self) -> bool {
        match self {
            SemaphoreInner::Counting { current, .. } => {
                if *current > 0 {
                    *current -= 1;
                    true
                } else {
                    false
                }
            }
            SemaphoreInner::Mutex {
                recursive,
                owner,
                lock_counter,
                ..
            } => {
                // In an ISR context we don't have a current task, so we can't implement
                // priority inheritance an we have to conjure up an owner.
                let current = NonNull::dangling();
                if let Some(owner) = owner {
                    if *owner == current && *recursive {
                        *lock_counter += 1;
                        true
                    } else {
                        false
                    }
                } else {
                    *owner = Some(current);
                    *lock_counter += 1;
                    true
                }
            }
        }
    }

    // 112
    fn try_give(&mut self) -> bool {
        match self {
            SemaphoreInner::Counting { current, max, .. } => {
                if *current < *max {
                    *current += 1;
                    true
                } else {
                    false
                }
            }
            SemaphoreInner::Mutex {
                owner,
                lock_counter,
                original_priority,
                ..
            } => SCHEDULER.with(|scheduler| {
                let current_cpu = Cpu::current() as usize;
                let current = unwrap!(scheduler.per_cpu[current_cpu].current_task);

                if *owner == Some(current) && *lock_counter > 0 {
                    *lock_counter -= 1;
                    if *lock_counter == 0
                        && let Some(owner) = owner.take()
                    {
                        trace!("Mutex set back to original priority");
                        owner.set_priority(&mut scheduler.run_queue, *original_priority);
                    }
                    true
                } else {
                    false
                }
            }),
        }
    }

    // 146
    fn try_give_from_isr(&mut self) -> bool {
        match self {
            SemaphoreInner::Counting { current, max, .. } => {
                if *current < *max {
                    *current += 1;
                    true
                } else {
                    false
                }
            }
            SemaphoreInner::Mutex {
                owner,
                lock_counter,
                ..
            } => {
                let current = NonNull::dangling();
                if *owner == Some(current) && *lock_counter > 0 {
                    *lock_counter -= 1;
                    if *lock_counter == 0 {
                        *owner = None;
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    // 175
    fn current_count(&mut self) -> u32 {
        match self {
            SemaphoreInner::Counting { current, .. } => *current,
            SemaphoreInner::Mutex { .. } => {
                panic!("RecursiveMutex does not support current_count")
            }
        }
    }

    // 184
    fn wait_with_deadline(&mut self, deadline: Instant) {
        match self {
            SemaphoreInner::Counting { waiting, .. } => waiting.wait_with_deadline(deadline),
            SemaphoreInner::Mutex { waiting, .. } => waiting.wait_with_deadline(deadline),
        }
    }

    // 192
    fn notify(&mut self) {
        trace!("semaphore notify");
        match self {
            SemaphoreInner::Counting { waiting, .. } => waiting.notify(),
            SemaphoreInner::Mutex { waiting, .. } => waiting.notify(),
        }
    }
}

/// Semaphore and mutex primitives.
// 202
pub struct Semaphore {
    inner: NonReentrantMutex<SemaphoreInner>,
}

// 206
unsafe impl Sync for Semaphore {}

// 208
impl Semaphore {
    /// Create a new counting semaphore.
    pub const fn new_counting(initial: u32, max: u32) -> Self {
        Semaphore {
            inner: NonReentrantMutex::new(SemaphoreInner::Counting {
                current: initial,
                max,
                waiting: WaitQueue::new(),
            }),
        }
    }

    /// Create a new mutex.
    ///
    /// If `recursive` is true, the mutex can be locked multiple times by the same task.
    // 223
    pub const fn new_mutex(recursive: bool) -> Self {
        Semaphore {
            inner: NonReentrantMutex::new(SemaphoreInner::Mutex {
                recursive,
                owner: None,
                lock_counter: 0,
                original_priority: Priority::new(0),
                waiting: WaitQueue::new(),
            }),
        }
    }

    /// Try to take the semaphore.
    ///
    /// This is a non-blocking operation. The return value indicates whether the semaphore was
    /// successfully taken.
    // 237
    pub fn try_take(&self) -> bool {
        self.inner.with(|sem| sem.try_take())
    }

    /// Try to take the semaphore from an ISR.
    ///
    /// This is a non-blocking operation. The return value indicates whether the semaphore was
    /// successfully taken.
    // 245
    pub fn try_take_from_isr(&self) -> bool {
        self.inner.with(|sem| sem.try_take_from_isr())
    }

    /// Take the semaphore.
    ///
    /// This is a blocking operation.
    ///
    /// If the semaphore is already taken, the task will be blocked until the semaphore is released.
    /// Recursive mutexes can be locked multiple times by the mutex owner task.
    // 257
    pub fn take(&self, timeout_us: Option<u32>) -> bool {
        if crate::with_deadline(timeout_us, |deadline| {
            self.inner.with(|sem| {
                if sem.try_take() {
                    true
                } else {
                    // The task will go to sleep when the above critical section is released.
                    sem.wait_with_deadline(deadline);
                    trace!("semaphore wait_with_deadline");
                    false
                }
            })
        }) {
            //debug!("Semaphore - take - success");
            true
        } else {
            debug!("Semaphore - take - timed out");
            false
        }
    }

    /// Return the current count of the semaphore.
    // 278
    pub fn current_count(&self) -> u32 {
        self.inner.with(|sem| sem.current_count())
    }

    /// Unlock the semaphore.
    // 281
    pub fn give(&self) -> bool {
        self.inner.with(|sem| {
            if sem.try_give() {
                sem.notify();
                true
            } else {
                false
            }
        })
    }

    /// Try to unlock the semaphore from an ISR.
    ///
    /// The return value indicates whether the semaphore was successfully unlocked.
    // 295
    pub fn try_give_from_isr(&self) -> bool {
        self.inner.with(|sem| {
            if sem.try_give_from_isr() {
                sem.notify();
                true
            } else {
                false
            }
        })
    }
}
