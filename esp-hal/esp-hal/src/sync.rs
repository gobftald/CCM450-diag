#[cfg(single_core)]
// 5
use core::cell::Cell;
use core::cell::UnsafeCell;

// 10
mod single_core {
    use core::sync::atomic::{compiler_fence, Ordering};

    /// Trait for single-core locks.
    // 16
    pub trait RawLock {
        unsafe fn enter(&self) -> critical_section::RawRestoreState;
        unsafe fn exit(&self, token: critical_section::RawRestoreState);
    }

    /// A lock that disables interrupts.
    // 72
    pub struct InterruptLock;

    // 74
    impl RawLock for InterruptLock {
        // 75
        unsafe fn enter(&self) -> critical_section::RawRestoreState {
            cfg_if::cfg_if! {
                if #[cfg(riscv)] {
                    let mut mstatus = 0u32;
                    core::arch::asm!("csrrci {0}, mstatus, 8", inout(reg) mstatus);
                    let token = ((mstatus & 0b1000) != 0) as critical_section::RawRestoreState;
                } else {
                    compile_error!("Unsupported architecture")
                }
            }

            // Ensure no subsequent memory accesses are reordered to before interrupts are
            // disabled.
            compiler_fence(Ordering::SeqCst);

            token
        }

        // 96
        unsafe fn exit(&self, token: critical_section::RawRestoreState) {
            // Ensure no preceeding memory accesses are reordered to after interrupts are
            // enabled.
            compiler_fence(Ordering::SeqCst);

            cfg_if::cfg_if! {
                if #[cfg(riscv)] {
                    if token != 0 {
                        esp_riscv_rt::riscv::interrupt::enable();
                    }
                } else {
                    compile_error!("Unsupported architecture")
                }
            }
        }
    }
}

// 176
#[cfg(riscv)]
// The restore state is a u8 that is casted from a bool, so it has a value of
// 0x00 or 0x01 before we add the reentry flag to it.
pub const REENTRY_FLAG: u8 = 1 << 7;

/// A generic lock that wraps [`single_core::RawLock`] and
/// [`multicore::AtomicLock`] and tracks whether the caller has locked
/// recursively.
/// We don't implement the 'multicore' part
// 193
struct GenericRawMutex<L: single_core::RawLock> {
    lock: L,
    #[cfg(multi_core)]
    inner: multicore::AtomicLock,
    #[cfg(single_core)]
    is_locked: Cell<bool>,
}

// 203
impl<L: single_core::RawLock> GenericRawMutex<L> {
    /// Create a new lock.
    pub const fn new(lock: L) -> Self {
        Self {
            lock,
            #[cfg(multi_core)]
            inner: multicore::AtomicLock::new(),
            #[cfg(single_core)]
            is_locked: Cell::new(false),
        }
    }

    /// Acquires the lock.
    ///
    // 223
    unsafe fn acquire(&self) -> critical_section::RawRestoreState {
        #[cfg(single_core)]
        {
            let mut tkn = unsafe { self.lock.enter() };
            let was_locked = self.is_locked.replace(true);
            if was_locked {
                tkn |= REENTRY_FLAG;
            }
            tkn
        }
    }

    /// Releases the lock.
    ///
    // 276
    unsafe fn release(&self, token: critical_section::RawRestoreState) {
        if token & REENTRY_FLAG == 0 {
            #[cfg(single_core)]
            self.is_locked.set(false);

            self.lock.exit(token)
        }
    }

    /// Runs the callback with this lock locked.
    ///
    /// Note that this function is not reentrant, calling it reentrantly will
    /// panic.
    // 292
    pub fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        let _token = LockGuard::new(self);
        f()
    }
}

/// A mutual exclusion primitive.
///
/// This lock disables interrupts on the current core while locked.
// 306
pub struct RawMutex {
    inner: GenericRawMutex<single_core::InterruptLock>,
}

// 316
impl RawMutex {
    /// Create a new lock.
    // 318
    pub const fn new() -> Self {
        Self {
            inner: GenericRawMutex::new(single_core::InterruptLock),
        }
    }

    /// Runs the callback with this lock locked.
    ///
    /// Note that this function is not reentrant, calling it reentrantly will
    /// panic.
    // 353
    pub fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        self.inner.lock(f)
    }
}

// Prefer this over a critical-section as this allows you to have multiple
// locks active at the same time rather than using the global mutex that is
// critical-section.
// 408
pub(crate) fn lock<T>(lock: &RawMutex, f: impl FnOnce() -> T) -> T {
    lock.lock(f)
}

/// Data protected by a [RawMutex].
/// We implement it only for wifi's concurent tasks running in its internal scheduler
///
/// This is largely equivalent to a `Mutex<RefCell<T>>`, but accessing the inner
/// data doesn't hold a critical section on multi-core systems.
/// But we don't implement multi-core
// 416
pub struct Locked<T> {
    lock_state: RawMutex,
    data: UnsafeCell<T>,
}

// 421
impl<T> Locked<T> {
    /// Create a new instance
    // 423
    pub const fn new(data: T) -> Self {
        Self {
            lock_state: RawMutex::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Provide exclusive access to the protected data to the given closure.
    ///
    /// Calling this reentrantly will panic.
    // 433
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        lock(&self.lock_state, || f(unsafe { &mut *self.data.get() }))
    }
}

// 456
struct LockGuard<'a, L: single_core::RawLock> {
    lock: &'a GenericRawMutex<L>,
    token: critical_section::RawRestoreState,
}

// 461
impl<'a, L: single_core::RawLock> LockGuard<'a, L> {
    fn new(lock: &'a GenericRawMutex<L>) -> Self {
        let this = Self::new_reentrant(lock);
        assert!(this.token & REENTRY_FLAG == 0, "lock is not reentrant");
        this
    }

    // 468
    fn new_reentrant(lock: &'a GenericRawMutex<L>) -> Self {
        let token = unsafe {
            // SAFETY: the same lock will be released when dropping the guard.
            // This ensures that the lock is released on the same thread, in the reverse
            // order it was acquired.
            lock.acquire()
        };

        Self { lock, token }
    }
}

// 480
impl<L: single_core::RawLock> Drop for LockGuard<'_, L> {
    fn drop(&mut self) {
        unsafe { self.lock.release(self.token) };
    }
}
