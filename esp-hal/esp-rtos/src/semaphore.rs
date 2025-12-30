//! Semaphores and mutexes.
//!
//! This module provides the [`Semaphore`] type, which implements counting semaphores and mutexes.

// 8
use esp_sync::NonReentrantMutex;

// 10
use crate::{run_queue::Priority, task::TaskPtr, wait_queue::WaitQueue};

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

/// Semaphore and mutex primitives.
// 202
pub struct Semaphore {
    inner: NonReentrantMutex<SemaphoreInner>,
}
