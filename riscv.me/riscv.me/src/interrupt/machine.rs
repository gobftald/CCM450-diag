use crate::{
    interrupt::Trap,
    register::{mepc, mstatus},
};

/// Disables interrupts globally in the current hart (machine mode).
// 105
#[inline]
pub fn disable() {
    // SAFETY: It is safe to disable interrupts
    unsafe { mstatus::clear_mie() }
}

/// Enables interrupts globally in the current hart (machine mode).
///
/// # Note
///
/// Only enabled interrupt sources will be triggered.
/// To enable specific interrupt sources, use [`enable_interrupt`].
///
/// # Safety
///
/// Enabling interrupts might break critical sections or other synchronization mechanisms.
/// Ensure that this is called in a safe context where interrupts can be enabled.
// 148
#[inline]
pub unsafe fn enable() {
    mstatus::set_mie()
}

/// Execute closure `f` with interrupts enabled in the current hart (machine mode).
///
/// This method is assumed to be called within an interrupt handler, and allows
/// nested interrupts to occur. After the closure `f` is executed, the [`mstatus`]
/// and [`mepc`] registers are properly restored to their previous values.
///
/// # Safety
///
/// - Do not call this function inside a critical section.
/// - This method is assumed to be called within an interrupt handler.
/// - Make sure to clear the interrupt flag that caused the interrupt before calling
///   this method. Otherwise, the interrupt will be re-triggered before executing `f`.
// 212
#[inline]
pub unsafe fn nested<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let mstatus = mstatus::read();
    let mepc = mepc::read();

    // enable interrupts to allow nested interrupts
    enable();

    let r = f();

    // If the interrupts were inactive before our `enable` call, then re-disable
    // them. Otherwise, keep them enabled
    if !mstatus.mie() {
        disable();
    }

    // Restore MSTATUS.PIE, MSTATUS.MPP, and SEPC
    if mstatus.mpie() {
        mstatus::set_mpie();
    }
    mstatus::set_mpp(mstatus.mpp());
    mepc::write(mepc);

    r
}
