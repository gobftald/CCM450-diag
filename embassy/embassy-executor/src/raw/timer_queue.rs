//! Timer queue operations.

use core::cell::Cell;

use super::TaskRef;

/// An item in the timer queue.
// 47
pub struct TimerQueueItem {
    /// The next item in the queue.
    ///
    /// If this field contains `Some`, the item is in the queue. The last item in the queue has a
    /// value of `Some(dangling_pointer)`
    pub next: Cell<Option<TaskRef>>,

    /// The time at which this item expires.
    pub expires_at: Cell<u64>,

    /// Some implementation-defined, zero-initialized piece of data.
    #[cfg(feature = "_timer-item-payload")]
    pub payload: OpaqueData,
}

// 62
//unsafe impl Sync for TimerQueueItem {}

// 64
impl TimerQueueItem {
    pub(crate) const fn new() -> Self {
        Self {
            next: Cell::new(None),
            expires_at: Cell::new(0),
            #[cfg(feature = "_timer-item-payload")]
            payload: OpaqueData::new(),
        }
    }
}
