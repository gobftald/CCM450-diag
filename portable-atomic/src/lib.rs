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

// 544
cfg_has_atomic_8! {
    /// A boolean type which can be safely shared between threads.
    ///
    /// This type has the same in-memory representation as a [`bool`].
    ///
    #[repr(C, align(1))]
    // 555
    pub struct AtomicBool {
        v: core::cell::UnsafeCell<u8>,
    }
}

// Send is implicitly implemented.
// SAFETY: any data races are prevented by disabling interrupts or
// atomic intrinsics (see module-level comments).
// 578
unsafe impl Sync for AtomicBool {}

// UnwindSafe is implicitly implemented.
#[cfg(not(portable_atomic_no_core_unwind_safe))]
impl core::panic::RefUnwindSafe for AtomicBool {}

// 588
impl AtomicBool {
    /// Creates a new `AtomicBool`.
    ///
    #[inline]
    #[must_use]
    // 601
    pub const fn new(v: bool) -> Self {
        static_assert_layout!(AtomicBool, bool);
        Self {
            v: core::cell::UnsafeCell::new(v as u8),
        }
    }

    /// Loads a value from the bool.
    ///
    /// `load` takes an [`Ordering`] argument which describes the memory ordering
    /// of this operation. Possible values are [`SeqCst`], [`Acquire`] and [`Relaxed`].
    // 766
    pub fn load(&self, order: Ordering) -> bool {
        self.as_atomic_u8().load(order) != 0
    }

    /// Stores a value into the bool.
    ///
    /// `store` takes an [`Ordering`] argument which describes the memory ordering
    /// of this operation. Possible values are [`SeqCst`], [`Release`] and [`Relaxed`].
    #[inline]
    // 794
    pub fn store(&self, val: bool, order: Ordering) {
        self.as_atomic_u8().store(val as u8, order);
    }

    #[inline(always)]
    // 1451
    fn as_atomic_u8(&self) -> &imp::AtomicU8 {
        // SAFETY: AtomicBool and imp::AtomicU8 have the same layout,
        // and both access data in the same way.
        unsafe { &*(self as *const Self as *const imp::AtomicU8) }
    }
}

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

            /// Stores a value into the atomic integer if the current value is the same as the `current` value.
            /// Unlike [`compare_exchange`](Self::compare_exchange) this function is allowed to spuriously fail even
            /// when the comparison succeeds, which can result in more efficient code on some platforms.
            /// The return value is a result indicating whether the new value was written and containing the
            /// previous value.
            #[inline]
            pub fn compare_exchange_weak(
                &self,
                current: $int_type,
                new: $int_type,
                success: Ordering,
                failure: Ordering,
            ) -> Result<$int_type, $int_type> {
                self.inner
                    .compare_exchange_weak(current, new, success, failure)
            }

            #[inline]
            // 3240
            pub fn fetch_add(&self, val: $int_type, order: Ordering) -> $int_type {
                self.inner.fetch_add(val, order)
            }

            /// Fetches the value, and applies a function to it that returns an optional
            /// new value. Returns a `Result` of `Ok(previous_value)` if the function returned `Some(_)`, else
            /// `Err(previous_value
            ///
            #[inline]
            // 3616
            pub fn fetch_update<F>(
                &self,
                set_order: Ordering,
                fetch_order: Ordering,
                mut f: F,
            ) -> Result<$int_type, $int_type>
            where
                F: FnMut($int_type) -> Option<$int_type>,
            {
                let mut prev = self.load(fetch_order);
                while let Some(next) = f(prev) {
                    match self.compare_exchange_weak(prev, next, set_order, fetch_order) {
                        x @ Ok(_) => return x,
                        Err(next_prev) => prev = next_prev,
                    }
                }
                Err(prev)
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
