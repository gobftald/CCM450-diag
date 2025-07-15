#![allow(dead_code)]

// On some platforms, atomic load/store can be implemented in a more efficient
// way than disabling interrupts. On MSP430, some RMWs that do not return the
// previous value can also be optimized.
//
// Note: On single-core systems, it is okay to use critical session-based
// CAS together with atomic load/store. The load/store will not be
// called while interrupts are disabled, and since the load/store is
// atomic, it is not affected by interrupts even if interrupts are enabled.
#[cfg(not(any(
    all(target_arch = "avr", portable_atomic_no_asm),
    feature = "critical-section",
)))]
// 48
use self::arch::atomic;

#[cfg_attr(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    path = "riscv.rs"
)]
// 69
mod arch;

// 71
use core::{cell::UnsafeCell, sync::atomic::Ordering};

#[cfg(not(feature = "critical-section"))]
#[inline(always)]
// 92
fn with<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // Get current interrupt state and disable interrupts
    let state = arch::disable();

    let r = f();

    // Restore interrupt state
    // SAFETY: the state was retrieved by the previous `disable`.
    unsafe { arch::restore(state) }

    r
}

// 249
macro_rules! atomic_int {
    // 250
    (base, $atomic_type:ident, $int_type:ident, $align:literal) => {
        #[repr(C, align($align))]
        // 252
        pub(crate) struct $atomic_type {
            v: UnsafeCell<$int_type>,
        }

        // Send is implicitly implemented.
        // SAFETY: any data races are prevented by disabling interrupts or
        // atomic intrinsics (see module-level comments).
        // 259
        unsafe impl Sync for $atomic_type {}

        // 261
        impl $atomic_type {
            #[inline]
            // 263
            pub(crate) const fn new(v: $int_type) -> Self {
                Self {
                    v: UnsafeCell::new(v),
                }
            }
        }
    };

    // 279
    (load_store_atomic $([$kind:ident])?, $atomic_type:ident, $int_type:ident, $align:literal) => {
        atomic_int!(base, $atomic_type, $int_type, $align);
        #[cfg(not(all(
            any(target_arch = "riscv32", target_arch = "riscv64"),
            not(feature = "critical-section"),
            any(
                portable_atomic_force_amo,
                target_feature = "zaamo",
                portable_atomic_target_feature = "zaamo",
            ),
        )))]
        atomic_int!(cas[emulate], $atomic_type, $int_type);

        // 301
        impl $atomic_type {
            #[inline]
            #[cfg_attr(
                all(debug_assertions, not(portable_atomic_no_track_caller)),
                track_caller
            )]
            // 304
            pub(crate) fn load(&self, order: Ordering) -> $int_type {
                crate::utils::assert_load_ordering(order);
                #[cfg(not(any(
                    all(target_arch = "avr", portable_atomic_no_asm),
                    feature = "critical-section",
                )))]
                {
                    self.as_native().load(order)
                }
            }

            #[inline]
            #[cfg_attr(
                all(debug_assertions, not(portable_atomic_no_track_caller)),
                track_caller
            )]
            // 325
            pub(crate) fn store(&self, val: $int_type, order: Ordering) {
                crate::utils::assert_store_ordering(order);
                #[cfg(not(any(
                    all(target_arch = "avr", portable_atomic_no_asm),
                    feature = "critical-section",
                )))]
                {
                    self.as_native().store(val, order);
                }
            }

            #[cfg(not(any(
                all(target_arch = "avr", portable_atomic_no_asm),
                feature = "critical-section",
            )))]
            #[inline(always)]
            // 349
            fn as_native(&self) -> &atomic::$atomic_type {
                // SAFETY: $atomic_type and atomic::$atomic_type have the same layout and
                // guarantee atomicity in a compatible way. (see module-level comments)
                unsafe { &*(self as *const Self as *const atomic::$atomic_type) }
            }
        }
    };

    // 426
    (cas[emulate], $atomic_type:ident, $int_type:ident) => {
        impl $atomic_type {
            #[inline]
            pub(crate) fn fetch_add(&self, val: $int_type, _order: Ordering) -> $int_type {
                // SAFETY: any data races are prevented by disabling interrupts (see
                // module-level comments) and the raw pointer is valid because we got it
                // from a reference.
                with(|| unsafe {
                    let prev = self.v.get().read();
                    self.v.get().write(prev.wrapping_add(val));
                    prev
                })
            }
        }
    };
}

#[cfg(not(target_pointer_width = "16"))]
// 904
atomic_int!(load_store_atomic, AtomicU32, u32, 4);
