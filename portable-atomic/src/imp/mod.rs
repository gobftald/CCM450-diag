// RISC-V without A-extension
#[cfg(any(test, not(feature = "critical-section")))]
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(any(
        all(test, not(any(miri, portable_atomic_sanitize_thread))),
        portable_atomic_no_atomic_cas,
    ))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(any(
        all(test, not(any(miri, portable_atomic_sanitize_thread))),
        not(target_has_atomic = "ptr"),
    ))
)]
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
// 67
mod riscv;

#[cfg(any(
    all(test, target_os = "none"),
    portable_atomic_unsafe_assume_single_core,
    feature = "critical-section",
    target_arch = "avr",
    target_arch = "msp430",
))]
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(any(test, portable_atomic_no_atomic_cas))
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(any(test, not(target_has_atomic = "ptr")))
)]
#[cfg(any(
    target_arch = "arm",
    target_arch = "avr",
    target_arch = "msp430",
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "xtensa",
    feature = "critical-section",
))]
// 181
mod interrupt;

// no core atomic CAS & (assume single core | critical section)
#[cfg(any(
    portable_atomic_unsafe_assume_single_core,
    feature = "critical-section",
    target_arch = "avr",
    target_arch = "msp430",
))]
#[cfg_attr(
    portable_atomic_no_cfg_target_has_atomic,
    cfg(portable_atomic_no_atomic_cas)
)]
#[cfg_attr(
    not(portable_atomic_no_cfg_target_has_atomic),
    cfg(not(target_has_atomic = "ptr"))
)]
items! {
    pub(crate) use self::interrupt::AtomicUsize;

    #[cfg(any(not(target_pointer_width = "16"), feature = "fallback"))]
    // 278
    pub(crate) use self::interrupt::AtomicU32;
}
