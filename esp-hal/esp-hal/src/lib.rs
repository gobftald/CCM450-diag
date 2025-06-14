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

//180
#![no_std]
#![feature(variant_count)]
#![allow(static_mut_refs)]

#[macro_use(assert, unreachable, panic, debug, unwrap)]
extern crate console;

// 206
pub use self::soc::peripherals;
pub(crate) use self::soc::peripherals::pac;

// 217
#[cfg(system)]
pub mod clock;

// 222
pub mod peripheral;

// 227
pub mod system;
pub mod time;

// 236
mod macros;

// 232
pub use procmacros::blocking_main as main;

// 288
pub mod config;

#[cfg(interrupt_core0)]
// 291
pub mod interrupt;
pub mod rom;

#[cfg(systimer)]
// 299
pub mod timer;

#[cfg(rtc_cntl)]
// 301
pub mod rtc_cntl;

#[cfg(rng)]
// 336
pub mod rng;

// The `soc` module contains chip-specific implementation details
// and should not be directly exposed.
// 365
mod soc;

/*
// 430
pub(crate) mod private {
    // 433
    pub trait Sealed {}
}
*/

// 508
pub mod __macro_implementation {
    #[cfg(riscv)]
    // 518
    pub use esp_riscv_rt::entry as __entry;
    #[cfg(xtensa)]
    // 520
    pub use xtensa_lx_rt::entry as __entry;
}

#[cfg(riscv)]
#[export_name = "hal_main"]
// 525
fn hal_main(a0: usize, a1: usize, a2: usize) -> ! {
    extern "Rust" {
        // This symbol will be provided by the user via `#[entry]`
        fn main(a0: usize, a1: usize, a2: usize) -> !;
    }

    unsafe {
        main(a0, a1, a2);
    }
}

// 554
use crate::config::WatchdogConfig;

// 555
use crate::{
    clock::{Clocks, CpuClock},
    peripherals::Peripherals,
};

/// System configuration.
///
/// This `struct` is marked with `#[non_exhaustive]` and can't be instantiated
/// directly. This is done to prevent breaking changes when new fields are added
/// to the `struct`. Instead, use the [`Config::default()`] method to create a
/// new instance.
///
/// For usage examples, see the [config module documentation](crate::config).
#[non_exhaustive]
#[derive(/*Default,*/ Clone, Copy /*procmacros::BuilderLite*/)]
// 570
pub struct Config {
    /// The CPU clock configuration.
    cpu_clock: CpuClock,

    /// Enable watchdog timer(s).
    _watchdog: WatchdogConfig,
}

impl Config {
    pub fn new_and_default(cpu_clock: CpuClock) -> Self {
        Self {
            cpu_clock,
            _watchdog: WatchdogConfig::default(),
        }
    }
}

/// Initialize the system.
///
/// This function sets up the CPU clock and watchdog, then, returns the
/// peripherals and clocks.
// 592
pub fn init(config: Config) -> Peripherals {
    // empty implementation
    //crate::soc::pre_init();

    system::disable_peripherals();

    let mut peripherals = Peripherals::take();

    // RTC domain must be enabled before we try to disable
    let mut rtc = crate::rtc_cntl::Rtc::new(peripherals.LPWR.reborrow());

    // Handle watchdog configuration with defaults
    #[cfg(not(feature = "unstable"))]
    {
        #[cfg(not(any(esp32, esp32s2)))]
        rtc.swd.disable();

        rtc.rwdt.disable();

        crate::timer::timg::Wdt::<crate::peripherals::TIMG0<'static>>::new().disable();
    }

    Clocks::init(config.cpu_clock);

    #[cfg(esp32)]
    crate::time::time_init();

    //crate::gpio::interrupt::bind_default_interrupt_handler();

    peripherals
}
