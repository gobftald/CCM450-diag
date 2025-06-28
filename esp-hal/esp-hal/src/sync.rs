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
                    //if token != 0 {
                    if token {
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
}
