pub mod machine {
    use crate::register::mstatus;

    /// Disables all interrupts in the current hart (machine mode).
    #[inline]
    // 10
    pub fn disable() {
        // SAFETY: It is safe to disable interrupts
        unsafe { mstatus::clear_mie() }
    }

    /// Enables all the interrupts in the current hart (machine mode).
    ///
    /// # Safety
    ///
    /// Do not call this function inside a critical section.
    #[inline]
    // 21
    pub unsafe fn enable() {
        mstatus::set_mie()
    }
}

#[cfg(not(feature = "s-mode"))]
pub use machine::*;
