use core::cell::UnsafeCell;

use esp_sync::RawMutex;

/// A non-reentrant (panicking) mutex.
///
/// This is largely equivalent to a `critical_section::Mutex<RefCell<T>>`, but accessing the inner
/// data doesn't hold a critical section on multi-core systems.
pub struct NonReentrantMutex<T> {
    lock_state: RawMutex,
    data: UnsafeCell<T>,
}

impl<T> NonReentrantMutex<T> {
    /// Create a new instance
    pub const fn new(data: T) -> Self {
        Self {
            lock_state: RawMutex::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Provide exclusive access to the protected data to the given closure.
    ///
    /// Calling this reentrantly will panic.
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.lock_state
            .lock_non_reentrant(|| f(unsafe { &mut *self.data.get() }))
    }
}

/*
unsafe impl<T: Send> Send for NonReentrantMutex<T> {}
unsafe impl<T: Send> Sync for NonReentrantMutex<T> {}
*/
unsafe impl<T> Sync for NonReentrantMutex<T> {}

#[cfg(impl_critical_section)]
#[cfg(feature = "rt")]
mod critical_section {
    struct CriticalSection;

    critical_section::set_impl!(CriticalSection);

    static CRITICAL_SECTION: esp_sync::RawMutex = esp_sync::RawMutex::new();

    unsafe impl critical_section::Impl for CriticalSection {
        unsafe fn acquire() -> critical_section::RawRestoreState {
            unsafe { CRITICAL_SECTION.acquire().inner() }
        }

        unsafe fn release(token: critical_section::RawRestoreState) {
            unsafe {
                CRITICAL_SECTION.release(esp_sync::RestoreState::new(token));
            }
        }
    }
}
