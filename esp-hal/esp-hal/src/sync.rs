// 4
#[cfg(single_core)]
use core::cell::Cell;
use core::cell::UnsafeCell;

/// Opaque token that can be used to release a lock.
// The interpretation of this value depends on the lock type that created it,
// but bit #31 is reserved for the reentry flag.
//
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 25
pub struct RestoreState(u32);

// 26
impl RestoreState {
    const REENTRY_FLAG: u32 = 1 << 31;

    fn mark_reentry(&mut self) {
        self.0 |= Self::REENTRY_FLAG;
    }

    fn is_reentry(&self) -> bool {
        self.0 & Self::REENTRY_FLAG != 0
    }
}

// 52
mod single_core {
    use core::sync::atomic::{compiler_fence, Ordering};

    // 55
    use super::RestoreState;

    /// Trait for single-core locks.
    // 59
    pub trait RawLock {
        unsafe fn enter(&self) -> RestoreState;
        unsafe fn exit(&self, token: RestoreState);
    }

    /// A lock that disables interrupts.
    // 114
    pub struct InterruptLock;

    // 116
    impl RawLock for InterruptLock {
        // 117
        unsafe fn enter(&self) -> RestoreState {
            cfg_if::cfg_if! {
                if #[cfg(riscv)] {
                    let mut mstatus = 0u32;
                    core::arch::asm!("csrrci {0}, mstatus, 8", inout(reg) mstatus);
                    let token = mstatus & 0b1000;
                } else {
                    compile_error!("Unsupported architecture")
                }
            }

            // Ensure no subsequent memory accesses are reordered to before interrupts are
            // disabled.
            compiler_fence(Ordering::SeqCst);

            RestoreState(token)
        }

        // 138
        unsafe fn exit(&self, token: RestoreState) {
            // Ensure no preceeding memory accesses are reordered to after interrupts are
            // enabled.
            compiler_fence(Ordering::SeqCst);

            let RestoreState(token) = token;

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

/// A generic lock that wraps [`single_core::RawLock`] and
/// [`multicore::AtomicLock`] and tracks whether the caller has locked
/// recursively.
/// We don't implement the 'multicore' part
// 226
struct GenericRawMutex<L: single_core::RawLock> {
    lock: L,
    #[cfg(multi_core)]
    inner: multicore::AtomicLock,
    #[cfg(single_core)]
    is_locked: Cell<bool>,
}

// 234
unsafe impl<L: single_core::RawLock> Sync for GenericRawMutex<L> {}

// 236
impl<L: single_core::RawLock> GenericRawMutex<L> {
    /// Create a new lock.
    // 238
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
    // 256
    unsafe fn acquire(&self) -> RestoreState {
        #[cfg(single_core)]
        {
            let mut tkn = unsafe { self.lock.enter() };
            let was_locked = self.is_locked.replace(true);
            if was_locked {
                tkn.mark_reentry();
            }
            tkn
        }
    }

    /// Releases the lock.
    ///
    /// - This function must only be called if the lock was acquired by the
    ///   current thread.
    /// - The caller must ensure to release the locks in the reverse order they
    ///   were acquired.
    /// - Each release call must be paired with an acquire call.
    // 309
    unsafe fn release(&self, token: RestoreState) {
        unsafe {
            if !token.is_reentry() {
                #[cfg(single_core)]
                self.is_locked.set(false);

                self.lock.exit(token)
            }
        }
    }

    /// Runs the callback with this lock locked.
    ///
    /// Note that this function is not reentrant, calling it reentrantly will
    /// panic.
    // 327
    pub fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        let _token = LockGuard::new(self);
        f()
    }
}

/// A mutual exclusion primitive.
///
/// This lock disables interrupts on the current core while locked.
// 341
pub struct RawMutex {
    inner: GenericRawMutex<single_core::InterruptLock>,
}

// 351
impl RawMutex {
    /// Create a new lock.
    // 353
    pub const fn new() -> Self {
        Self {
            inner: GenericRawMutex::new(single_core::InterruptLock),
        }
    }

    /// Acquires the lock.
    ///
    // 367
    pub unsafe fn acquire(&self) -> RestoreState {
        unsafe { self.inner.acquire() }
    }

    /// Releases the lock.
    ///
    // 380
    pub unsafe fn release(&self, token: RestoreState) {
        unsafe {
            self.inner.release(token);
        }
    }

    /// Runs the callback with this lock locked.
    ///
    /// Note that this function is not reentrant, calling it reentrantly will
    /// panic.
    // 390
    pub fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        self.inner.lock(f)
    }
}

// 395
unsafe impl embassy_sync::blocking_mutex::raw::RawMutex for RawMutex {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self::new();

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        // embassy_sync semantics allow reentrancy.
        let _token = LockGuard::new_reentrant(&self.inner);
        f()
    }
}

// Prefer this over a critical-section as this allows you to have multiple
// locks active at the same time rather than using the global mutex that is
// critical-section.
// 445
pub(crate) fn lock<T>(lock: &RawMutex, f: impl FnOnce() -> T) -> T {
    lock.lock(f)
}

/// Data protected by a [RawMutex].
/// We implement it only for wifi's concurent tasks running in its internal scheduler
///
/// This is largely equivalent to a `Mutex<RefCell<T>>`, but accessing the inner
/// data doesn't hold a critical section on multi-core systems.
/// But we don't implement multi-core
// 453
pub struct Locked<T> {
    lock_state: RawMutex,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for Locked<T> {}

// 458
impl<T> Locked<T> {
    /// Create a new instance
    // 460
    pub const fn new(data: T) -> Self {
        Self {
            lock_state: RawMutex::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Provide exclusive access to the protected data to the given closure.
    ///
    /// Calling this reentrantly will panic.
    // 470
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        lock(&self.lock_state, || f(unsafe { &mut *self.data.get() }))
    }
}

// 477
struct LockGuard<'a, L: single_core::RawLock> {
    lock: &'a GenericRawMutex<L>,
    token: RestoreState,
}

// 482
impl<'a, L: single_core::RawLock> LockGuard<'a, L> {
    // 483
    fn new(lock: &'a GenericRawMutex<L>) -> Self {
        let this = Self::new_reentrant(lock);
        assert!(!this.token.is_reentry(), "lock is not reentrant");
        this
    }

    // 489
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

// 501
impl<L: single_core::RawLock> Drop for LockGuard<'_, L> {
    fn drop(&mut self) {
        unsafe { self.lock.release(self.token) };
    }
}
