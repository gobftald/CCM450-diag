#![no_std]

//! ## Implementing a driver
//!
//! - Define a struct `MyDriver`
//! - Implement [`Driver`] for it
//! - Register it as the global driver with [`time_driver_impl`](crate::time_driver_impl).
//!
//! If your driver has a single set tick rate, enable the corresponding [`tick-hz-*`](crate#tick-rate) feature,
//! which will prevent users from needing to configure it themselves (or selecting an incorrect configuration).
//!
//! If your driver supports a small number of set tick rates, expose your own cargo features and have each one
//! enable the corresponding `embassy-time-driver/tick-*`.
//!
//! Otherwise, don’t enable any `tick-hz-*` feature to let the user configure the tick rate themselves by
//! enabling a feature on `embassy-time`.
//!
//! ### Example
//!
//! ```
//! use core::task::Waker;
//!
//! use embassy_time_driver::Driver;
//!
//! struct MyDriver{} // not public!
//!
//! impl Driver for MyDriver {
//!     fn now(&self) -> u64 {
//!         todo!()
//!     }
//!
//!     fn schedule_wake(&self, at: u64, waker: &Waker) {
//!         todo!()
//!     }
//! }
//!
//! embassy_time_driver::time_driver_impl!(static DRIVER: MyDriver = MyDriver{});
//! ```
//!
//! # Linkage details
//!
//! Instead of the usual "trait + generic params" approach, calls from embassy to the driver are done via `extern` functions.
//!
//! `embassy` internally defines the driver function as `extern "Rust" { fn _embassy_time_now() -> u64; }` and calls it.
//! The driver crate defines the function as `#[no_mangle] fn _embassy_time_now() -> u64`. The linker will resolve the
//! calls from the `embassy` crate to call into the driver crate.
//!
//! If there is none or multiple drivers in the crate tree, linking will fail.
//!
//! This method has a few key advantages for something as foundational as timekeeping:
//!
//! - The time driver is available everywhere easily, without having to thread the implementation
//!   through generic parameters. This is especially helpful for libraries.
//! - It means comparing `Instant`s will always make sense: if there were multiple drivers
//!   active, one could compare an `Instant` from driver A to an `Instant` from driver B, which
//!   would yield incorrect results.

// 110
mod tick;

/// Ticks per second of the global timebase.
///
/// This value is specified by the [`tick-*` Cargo features](crate#tick-rate)
// 115
pub const TICK_HZ: u64 = tick::TICK_HZ;

/// Time driver
// 118
//pub trait Driver: Send + Sync + 'static
pub trait Driver: 'static {
    /// Return the current timestamp in ticks.
    ///
    /// Implementations MUST ensure that:
    /// - This is guaranteed to be monotonic, i.e. a call to now() will always return
    ///   a greater or equal value than earlier calls. Time can't "roll backwards".
    /// - It "never" overflows. It must not overflow in a sufficiently long time frame, say
    ///   in 10_000 years (Human civilization is likely to already have self-destructed
    ///   10_000 years from now.). This means if your hardware only has 16bit/32bit timers
    ///   you MUST extend them to 64-bit, for example by counting overflows in software,
    ///   or chaining multiple timers together.
    fn now(&self) -> u64;
}

// 136
extern "Rust" {
    fn _embassy_time_now() -> u64;
}

/// See [`Driver::now`]
#[inline]
// 143
pub fn now() -> u64 {
    unsafe { _embassy_time_now() }
}

/// Set the time Driver implementation.
///
/// See the module documentation for an example.
#[macro_export]
// 157
macro_rules! time_driver_impl {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[no_mangle]
        #[inline]
        fn _embassy_time_now() -> u64 {
            <$t as $crate::Driver>::now(&$name)
        }
    };
}

// called from 'esp-hal-embassy/src/time_driver.rs'
// embassy_time_driver::time_driver_impl!(static DRIVER: EmbassyTimer = EmbassyTimer {...}
// expanded to:
//  $name = DRIVER
//  $t = EmbassyTimer
//  $val = EmbassyTimer {...}
//
// static DRIVER: EmbassyTimer = EmbassyTimer {...};
//
// #[no_mangle]
// #[inline]
// fn _embassy_time_now() -> u64 {
//     <EmbassyTimer as $crate::Driver>::now(&DRIVER)
// }
