use crate::register::mstatus;

/// Enables all the interrupts in the current hart (machine mode).
///
/// # Safety
///
/// Do not call this function inside a critical section.
#[inline]
// 112
pub unsafe fn enable() {
    mstatus::set_mie()
}
