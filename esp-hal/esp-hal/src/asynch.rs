//! Asynchronous utilities.
use core::task::Waker;

// 4
use embassy_sync::waitqueue::GenericAtomicWaker;

// 6
use crate::sync::RawMutex;

/// Utility struct to register and wake a waker.
// 9
pub struct AtomicWaker {
    waker: GenericAtomicWaker<RawMutex>,
}

// 13
impl AtomicWaker {
    /// Create a new `AtomicWaker`.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            waker: GenericAtomicWaker::new(RawMutex::new()),
        }
    }

    delegate::delegate! {
        to self.waker {
            /// Register a waker. Overwrites the previous waker, if any.
            pub fn register(&self, w: &Waker);
            /// Wake the registered waker, if any.
            pub fn wake(&self);
        }
    }
}
