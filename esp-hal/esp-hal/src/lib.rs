//! # Bare-metal (`no_std`) HAL for all Espressif ESP32 devices.
//!
//! ## Choosing a Device
//!
//! Depending on your target device, you need to enable the chip feature
//! for that device. You may also need to do this on ancillary esp-hal crates.
//!
//! ## Overview
//!
//! ### Peripheral drivers
//!
//! The HAL implements both blocking _and_ async APIs for many peripherals.
//! Where applicable, driver implement the [embedded-hal] and
//! [embedded-hal-async] traits.
//!
//! ### Peripheral singletons
//!
//! Each peripheral driver needs a peripheral singleton that tells the driver
//! which hardware block to use. The peripheral singletons are created by the
//! HAL initialization, and are returned from [`init`] as fields of the
//! [`Peripherals`] struct.
//!
//! These singletons, by default, represent peripherals for the entire lifetime
//! of the program. To allow for reusing peripherals, the HAL provides a
//! `reborrow` method on each peripheral singleton. This method creates a new
//! handle to the peripheral with a shorter lifetime. This allows you to pass
//! the handle to a driver, while still keeping the original handle alive. Once
//! you drop the driver, you will be able to reborrow the peripheral again.
//!
//! For example, if you want to use the [`I2c`](i2c::master::I2c) driver and you
//! don't intend to drop the driver, you can pass the peripheral singleton to
//! the driver by value:
//!
//! ```rust, ignore
//! // Peripheral singletons are returned from the `init` function.
//! let peripherals = esp_hal::init(esp_hal::Config::default());
//!
//! let mut i2c = I2C::new(peripherals.I2C0, /* ... */);
//! ```
//!
//! If you want to use the peripheral in multiple places (for example, you want
//! to drop the driver for some period of time to minimize power consumption),
//! you can reborrow the peripheral singleton and pass it to the driver by
//! reference:
//!
//! ```rust, ignore
//! // Note that in this case, `peripherals` needs to be mutable.
//! let mut peripherals = esp_hal::init(esp_hal::Config::default());
//!
//! let i2c = I2C::new(peripherals.I2C0.reborrow(), /* ... */);
//!
//! // Do something with the I2C driver...
//!
//! core::mem::drop(i2c); // Drop the driver to minimize power consumption.
//!
//! // Do something else...
//!
//! // You can then take or reborrow the peripheral singleton again.
//! let i2c = I2C::new(peripherals.I2C0.reborrow(), /* ... */);
//! ```
//!
//! ## Don't use `core::mem::forget`
//!
//! You should never use `core::mem::forget` on any type defined in the HAL.
//! Some types heavily rely on their `Drop` implementation to not leave the
//! hardware in undefined state and causing UB.
//!

//182
#![no_std]

#[macro_use(assert, panic)]
extern crate console;

// 210
pub use self::soc::peripherals;
pub(crate) use self::soc::peripherals::pac;

// 220
#[cfg(any(/*dport, hp_sys, pcr,*/ system))]
pub mod clock;

// 226
pub mod peripheral;

// 232
pub mod time;

// 236
pub use procmacros::blocking_main as main;

#[cfg(any(/*dport,*/ interrupt_core0, /*interrupt_core1*/))]
// 293
pub mod interrupt;

// 302
#[cfg(any(systimer/* , timg0, timg1*/))]
pub mod timer;

// 369
// The `soc` module contains chip-specific implementation details
// and should not be directly exposed.
mod soc;

// 434
pub(crate) mod private {
    // 437
    pub trait Sealed {}
}

#[doc(hidden)]
// 486
pub mod __macro_implementation {
    #[cfg(riscv)]
    // 496
    pub use esp_riscv_rt::entry as __entry;
    #[cfg(xtensa)]
    // 498
    pub use xtensa_lx_rt::entry as __entry;
}

#[cfg(riscv)]
#[export_name = "hal_main"]
// 503
fn hal_main(a0: usize, a1: usize, a2: usize) -> ! {
    extern "Rust" {
        // This symbol will be provided by the user via `#[entry]`
        fn main(a0: usize, a1: usize, a2: usize) -> !;
    }

    unsafe {
        main(a0, a1, a2);
    }
}
