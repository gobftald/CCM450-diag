// in our (esp32c3) case:
// portable_atomic_unstable_asm =  false
// portable_atomic_no_atomic_load_store = false
// portable_atomic_no_cfg_target_has_atomic = false
// portable_atomic_no_atomic_cas = false
// portable_atomic_no_asm = false
// portable_atomic_no_track_caller = false
// target_pointer_width = "32"

#![no_std]

// 341
#[cfg(not(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64",
)))]
compile_error!(
    "portable-atomic currently only supports targets with {16,32,64}-bit pointer width; \
     if you need support for others, \
     please submit an issue at <https://github.com/taiki-e/portable-atomic>"
);

// 429
#[cfg(all(
    portable_atomic_unsafe_assume_single_core,
    feature = "critical-section"
))]
compile_error!(
    "you may not enable `critical-section` feature and `portable_atomic_unsafe_assume_single_core` cfg (`unsafe-assume-single-core` feature) at the same time"
);

#[macro_use]
// 465
mod cfgs;

#[cfg(target_pointer_width = "32")]
// 469
pub use self::cfg_has_atomic_32 as cfg_has_atomic_ptr;

#[macro_use]
// 476
mod utils;

// 483
pub use core::sync::atomic::Ordering;

// 492
mod imp;

// 2720
macro_rules! atomic_int {
    // Atomic{I,U}* impls
    ($atomic_type:ident, $int_type:ident, $align:literal) => {
        /// An integer type which can be safely shared between threads.
        /// This type has the same in-memory representation as the underlying integer type
        /// If the compiler and the platform support atomic loads and stores
        /// this type is a wrapper for the standard library's
        /// If the platform supports it but the compiler does not, atomic operations are implemented using
        /// inline assembly. Otherwise synchronizes using global locks.
        /// You can call is_lock_free() to check whether atomic instructions or locks will be used.

        // We can use #[repr(transparent)] here, but #[repr(C, align(N))]
        // will show clearer docs.
        #[repr(C, align($align))]
        // 2743
        pub struct $atomic_type {
            inner: imp::$atomic_type,
        }

        // 2770
        impl $atomic_type {
            /// Creates a new atomic integer.
            #[inline]
            #[must_use]
            // 2785
            pub const fn new(v: $int_type) -> Self {
                static_assert_layout!($atomic_type, $int_type);
                Self {
                    inner: imp::$atomic_type::new(v),
                }
            }

            /// Loads a value from the atomic integer.
            ///
            /// `load` takes an [`Ordering`] argument which describes the memory ordering of this operation.
            /// Possible values are [`SeqCst`], [`Acquire`] and [`Relaxed`].
            // 3044
            pub fn load(&self, order: Ordering) -> $int_type {
                self.inner.load(order)
            }

            /// Stores a value into the atomic integer.
            ///
            /// `store` takes an [`Ordering`] argument which describes the memory ordering of this operation.
            /// Possible values are [`SeqCst`], [`Release`] and [`Relaxed`].
            // 3074
            pub fn store(&self, val: $int_type, order: Ordering) {
                self.inner.store(val, order)
            }

            /// Adds to the current value, returning the previous value.
            ///
            /// This operation wraps around on overflow.
            ///
            /// `fetch_add` takes an [`Ordering`] argument which describes the memory ordering
            /// of this operation. All ordering modes are possible. Note that using
            /// [`Acquire`] makes the store part of this operation [`Relaxed`], and
            /// using [`Release`] makes the load part [`Relaxed`].

            #[inline]
            // 3240
            pub fn fetch_add(&self, val: $int_type, order: Ordering) -> $int_type {
                self.inner.fetch_add(val, order)
            }
        }
    };
}

// 4776
cfg_has_atomic_ptr! {
    #[cfg(target_pointer_width = "32")]
    atomic_int!(AtomicUsize, usize, 4);
}

// 4804
cfg_has_atomic_32! {
    atomic_int!(AtomicU32, u32, 4);
}
