//! Assembly instructions

macro_rules! instruction {
    ($(#[$attr:meta])*, unsafe $fnname:ident, $asm:expr, $($options:tt)*) => (
        $(#[$attr])*
        #[inline(always)]
        pub unsafe fn $fnname() {
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            core::arch::asm!($asm, $($options)*);
            #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
            unimplemented!();
        }
    );
    ($(#[$attr:meta])*, $fnname:ident, $asm:expr, $($options:tt)*) => (
        $(#[$attr])*
        #[inline(always)]
        pub fn $fnname() {
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            unsafe { core::arch::asm!($asm, $($options)*) };
            #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
            unimplemented!();
        }
    )
}

// 26
instruction!(
    /// `NOP` instruction wrapper
    ///
    /// The `NOP` instruction does not change any architecturally visible state, except for
    /// advancing the PC and incrementing any applicable performance counters.
    ///
    /// This function generates a no-operation; it's useful to prevent delay loops from being
    /// optimized away.
    , nop, "nop", options(nomem, nostack, preserves_flags));
