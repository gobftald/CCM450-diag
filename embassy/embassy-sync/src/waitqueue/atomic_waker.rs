use core::cell::Cell;
use core::task::Waker;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;

/// Utility struct to register and wake a waker.
/// If a waker is registered, registering another waker will replace the previous one without waking it.
/// Intended to wake a task from an interrupt. Therefore, it is generally not expected,
/// that multiple tasks register try to register a waker simultaneously.
// 11
pub struct GenericAtomicWaker<M: RawMutex> {
    waker: Mutex<M, Cell<Option<Waker>>>,
}

// 15
impl<M: RawMutex> GenericAtomicWaker<M> {
    /// Create a new `AtomicWaker`.
    // 17
    pub const fn new(mutex: M) -> Self {
        Self {
            waker: Mutex::const_new(mutex, Cell::new(None)),
        }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    // 24
    pub fn register(&self, w: &Waker) {
        self.waker.lock(|cell| {
            cell.set(match cell.replace(None) {
                Some(w2) if (w2.will_wake(w)) => Some(w2),
                _ => Some(w.clone()),
            })
        })
    }

    /// Wake the registered waker, if any.
    // 34
    pub fn wake(&self) {
        self.waker.lock(|cell| {
            if let Some(w) = cell.replace(None) {
                w.wake_by_ref();
                cell.set(Some(w));
            }
        })
    }
}
