//! Blocking mutex.
//!
//! This module provides a blocking mutex that can be used to synchronize data.
pub mod raw;

// 6
use core::cell::UnsafeCell;

// 8
use self::raw::RawMutex;

/// Blocking mutex (not async)
///
/// Provides a blocking mutual exclusion primitive backed by an implementation of [`raw::RawMutex`].
///
/// Which implementation you select depends on the context in which you're using the mutex, and you can choose which kind
/// of interior mutability fits your use case.
///
/// Use [`CriticalSectionMutex`] when data can be shared between threads and interrupts.
///
/// Use [`NoopMutex`] when data is only shared between tasks running on the same executor.
///
/// Use [`ThreadModeMutex`] when data is shared between tasks running on the same executor but you want a global singleton.
///
/// In all cases, the blocking mutex is intended to be short lived and not held across await points.
/// Use the async [`Mutex`](crate::mutex::Mutex) if you need a lock that is held across await points.
///
/// We can also use a smart/reentrant CriticalSectionMutex like RawMutex implementing in  esp_hal::sync::RawMutex

#[derive(Debug)]
// 26
pub struct Mutex<R, T: ?Sized> {
    // NOTE: `raw` must be FIRST, so when using ThreadModeMutex the "can't drop in non-thread-mode" gets
    // to run BEFORE dropping `data`.
    raw: R,
    data: UnsafeCell<T>,
}

// 32
unsafe impl<R: RawMutex + Send, T: ?Sized + Send> Send for Mutex<R, T> {}
unsafe impl<R: RawMutex + Sync, T: ?Sized + Send> Sync for Mutex<R, T> {}

// 35
impl<R: RawMutex, T> Mutex<R, T> {
    /// Creates a new mutex in an unlocked state ready for use.
    #[inline]
    // 39
    pub const fn new(val: T) -> Mutex<R, T> {
        Mutex {
            raw: R::INIT,
            data: UnsafeCell::new(val),
        }
    }

    /// Creates a critical section and grants temporary access to the protected data.
    // 46
    pub fn lock<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.raw.lock(|| {
            let ptr = self.data.get() as *const T;
            let inner = unsafe { &*ptr };
            f(inner)
        })
    }
}

// 72
impl<R, T> Mutex<R, T> {
    /// Creates a new mutex based on a pre-existing raw mutex.
    ///
    /// This allows creating a mutex in a constant context on stable Rust.
    #[inline]
    pub const fn const_new(raw_mutex: R, val: T) -> Mutex<R, T> {
        Mutex {
            raw: raw_mutex,
            data: UnsafeCell::new(val),
        }
    }
}
