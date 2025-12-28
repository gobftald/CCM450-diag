#[cfg(not(portable_atomic_no_asm))]
//18
use core::arch::asm;

// 20
pub(super) use super::super::riscv as atomic;

// Status register
#[cfg(not(portable_atomic_s_mode))]
// 24
macro_rules! status {
    () => {
        "mstatus"
    };
}

// MIE (Machine Interrupt Enable) bit (1 << 3)
#[cfg(not(portable_atomic_s_mode))]
const MASK: State = 0x8;
#[cfg(not(portable_atomic_s_mode))]
// 40
macro_rules! mask {
    () => {
        "0x8"
    };
}

#[cfg(target_arch = "riscv32")]
// 56
pub(super) type State = u32;

/// Disables interrupts and returns the previous interrupt state.
#[inline(always)]
pub(super) fn disable() -> State {
    let status: State;
    // SAFETY: reading mstatus/sstatus and disabling interrupts is safe.
    // (see module-level comments of interrupt/mod.rs on the safety of using privileged instructions)
    unsafe {
        // Do not use `nomem` and `readonly` because prevent subsequent memory accesses from being reordered before interrupts are disabled.
        asm!(
            concat!("csrrci {status}, ", status!(), ", ", mask!()), // atomic { status = status!(); status!() &= !mask!() }
            status = out(reg) status,
            options(nostack, preserves_flags),
        );
    }
    status
}

/// Restores the previous interrupt state.
///
/// # Safety
///
/// The state must be the one retrieved by the previous `disable`.
#[inline(always)]
// 83
pub(super) unsafe fn restore(status: State) {
    if status & MASK != 0 {
        // SAFETY: the caller must guarantee that the state was retrieved by the previous `disable`,
        // and we've checked that interrupts were enabled before disabling interrupts.
        unsafe {
            // Do not use `nomem` and `readonly` because prevent preceding memory accesses from being reordered after interrupts are enabled.
            asm!(
                concat!("csrsi ", status!(), ", ", mask!()), // atomic { status!() |= mask!() }
                options(nostack, preserves_flags),
            );
        }
    }
}
