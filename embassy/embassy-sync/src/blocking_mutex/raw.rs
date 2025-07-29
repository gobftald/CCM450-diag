//! Mutex primitives.
//!
//! This module provides a trait for mutexes that can be used in different contexts.

/// Raw mutex trait.
///
/// This mutex is "raw", which means it does not actually contain the protected data, it
/// just implements the mutex mechanism. For most uses you should use [`super::Mutex`] instead,
/// which is generic over a RawMutex and contains the protected data.
///
/// Note that, unlike other mutexes, implementations only guarantee no
/// concurrent access from other threads: concurrent access from the current
/// thread is allowed. For example, it's possible to lock the same mutex multiple times reentrantly.
///
/// Therefore, locking a `RawMutex` is only enough to guarantee safe shared (`&`) access
/// to the data, it is not enough to guarantee exclusive (`&mut`) access.
///
/// # Safety
///
/// RawMutex implementations must ensure that, while locked, no other thread can lock
/// the RawMutex concurrently.
///
/// Unsafe code is allowed to rely on this fact, so incorrect implementations will cause undefined behavior.
// 25
pub unsafe trait RawMutex {
    /// Create a new `RawMutex` instance.
    ///
    /// This is a const instead of a method to allow creating instances in const context.
    const INIT: Self;

    /// Lock this `RawMutex`.
    fn lock<R>(&self, f: impl FnOnce() -> R) -> R;
}

// we will use esp_hal::sync::RawMutex
