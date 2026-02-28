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

// MUST be the first module
mod fmt;

mod exception_handler;

//#[macro_use(assert, unreachable, panic, debug, info, unwrap)]
//extern crate console;

// 274
use core::marker::PhantomData;

#[macro_use]
extern crate esp_metadata_generated;

pub use esp_riscv_rt::riscv;

#[cfg(riscv)]
// 192
use esp_sync::RawMutex;

//#[cfg(efuse)]
// 201
pub use self::soc::efuse;

// 206
//pub use self::soc::peripherals;
//pub(crate) use self::soc::peripherals::pac;
pub(crate) use peripherals::pac;

//#[cfg(system)]
#[cfg(soc_has_system)]
pub mod clock;

#[cfg(gpio)]
// 219
pub mod gpio;

//pub mod peripheral;
pub mod peripherals;

// 227
pub mod system;
pub mod time;

// 232
mod macros;

// 230
//#[cfg(any(uart0, uart1))]
//#[cfg(uart1)]
#[cfg(soc_has_uart1)]
pub mod uart;

// 234
pub use procmacros::blocking_main as main;

// 242
pub use procmacros::{handler, ram};

// 287
pub mod asynch;
pub mod config;

// 294
pub mod sync;

//#[cfg(interrupt_core0)]
#[cfg(soc_has_interrupt_core0)]
// 291
pub mod interrupt;
pub mod rom;

#[cfg(systimer)]
// 299
pub mod timer;

//#[cfg(rtc_cntl)]
#[cfg(soc_has_lpwr)]
// 301
pub mod rtc_cntl;

#[cfg(rng)]
// 336
pub mod rng;

/// State of the CPU saved when entering exception or interrupt
// 404
#[cfg(feature = "rt")]
#[allow(unused_imports)]
pub mod trapframe {
    #[cfg(riscv)]
    pub use esp_riscv_rt::TrapFrame;
    #[cfg(xtensa)]
    pub use xtensa_lx_rt::exception::Context as TrapFrame;
}

// The `soc` module contains chip-specific implementation details
// and should not be directly exposed.
// 365
mod soc;

// 417
#[cfg(is_debug_build)]
procmacros::warning! {"
WARNING: use --release
  We *strongly* recommend using release profile when building esp-hal.
  The dev profile can potentially be one or more orders of magnitude
  slower than release, and may cause issues with timing-senstive
  peripherals and/or devices.
"}

/// A marker trait for driver modes.
///
/// Different driver modes offer different features and different API. Using
/// this trait as a generic parameter ensures that the driver is initialized in
/// the correct mode.
// 431
pub trait DriverMode: crate::private::Sealed {}

/// Marker type signalling that a driver is initialized in blocking mode.
///
/// Drivers are constructed in blocking mode by default. To learn about the
/// differences between blocking and async drivers, see the [`Async`] mode
/// documentation.
///
/// [`Async`] drivers can be converted to a [`Blocking`] driver using the
/// `into_blocking` method, for example:
///
/// ```rust, no_run
/// # {before_snippet}
/// # use esp_hal::uart::{Config, Uart};
/// let uart = Uart::new(peripherals.UART0, Config::default())?
///     .with_rx(peripherals.GPIO1)
///     .with_tx(peripherals.GPIO2)
///     .into_async();
///
/// let blocking_uart = uart.into_blocking();
///
/// # {after_snippet}
/// ```
// 455
#[derive(Debug)]
#[non_exhaustive]
pub struct Blocking;

/// Marker type signalling that a driver is initialized in async mode.
///
/// Drivers are constructed in blocking mode by default. To set up an async
/// driver, a [`Blocking`] driver must be converted to an `Async` driver using
/// the `into_async` method, for example:
///
/// ```rust, no_run
/// # {before_snippet}
/// # use esp_hal::uart::{Config, Uart};
/// let uart = Uart::new(peripherals.UART0, Config::default())?
///     .with_rx(peripherals.GPIO1)
///     .with_tx(peripherals.GPIO2)
///     .into_async();
///
/// # {after_snippet}
/// ```
///
/// Drivers can be converted back to blocking mode using the `into_blocking`
/// method, see [`Blocking`] documentation for more details.
///
/// Async mode drivers offer most of the same features as blocking drivers, but
/// with the addition of async APIs. Interrupt-related functions are not
/// available in async mode, as they are handled by the driver's interrupt
/// handlers.
///
/// Note that async functions usually take up more space than their blocking
/// counterparts, and they are generally slower. This is because async functions
/// are implemented using a state machine that is driven by interrupts and is
/// polled by a runtime. For short operations, the overhead of the state machine
/// can be significant. Consider using the blocking functions on the async
/// driver for small transfers.
///
/// When initializing an async driver, the driver disables user-specified
/// interrupt handlers, and sets up internal interrupt handlers that drive the
/// driver's async API. The driver's interrupt handlers run on the same core as
/// the driver was initialized on. This means that the driver can not be sent
/// across threads, to prevent incorrect concurrent access to the peripheral.
///
/// Switching back to blocking mode will disable the interrupt handlers and
/// return the driver to a state where it can be sent across threads.
// 500
#[derive(Debug)]
#[non_exhaustive]
pub struct Async(PhantomData<*const ()>);

// 504
unsafe impl Sync for Async {}

// 506
impl crate::DriverMode for Blocking {}
impl crate::DriverMode for Async {}
impl crate::private::Sealed for Blocking {}
impl crate::private::Sealed for Async {}

// 511
pub(crate) mod private {
    use core::mem::ManuallyDrop;

    // 514
    pub trait Sealed {}

    // 538
    pub(crate) struct OnDrop<F: FnOnce()>(ManuallyDrop<F>);
    // 458
    impl<F: FnOnce()> OnDrop<F> {
        pub fn new(cb: F) -> Self {
            Self(ManuallyDrop::new(cb))
        }
    }

    // 468
    impl<F: FnOnce()> Drop for OnDrop<F> {
        fn drop(&mut self) {
            unsafe { (ManuallyDrop::take(&mut self.0))() }
        }
    }
}

// 508
pub mod __macro_implementation {
    #[cfg(riscv)]
    // 518
    pub use esp_riscv_rt::entry as __entry;
    #[cfg(xtensa)]
    // 520
    pub use xtensa_lx_rt::entry as __entry;
}

// 554
//use crate::config::WatchdogConfig;

// 555
use crate::{
    clock::{Clocks, CpuClock},
    peripherals::Peripherals,
};

/// A spinlock for seldom called stuff. Users assume that lock contention is not an issue.
pub(crate) static ESP_HAL_LOCK: RawMutex = RawMutex::new();

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
    /*
    /// Enable watchdog timer(s).
    _watchdog: WatchdogConfig,
    */
}

impl Config {
    pub fn new_and_default(cpu_clock: CpuClock) -> Self {
        Self {
            cpu_clock,
            //_watchdog: WatchdogConfig::default(),
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
    //#[cfg(not(feature = "unstable"))]
    //{
    #[cfg(not(any(esp32, esp32s2)))]
    rtc.swd.disable();

    rtc.rwdt.disable();

    crate::timer::timg::Wdt::<crate::peripherals::TIMG0<'static>>::new().disable();
    //}

    Clocks::init(config.cpu_clock);

    #[cfg(esp32)]
    crate::time::time_init();

    //crate::gpio::interrupt::bind_default_interrupt_handler();

    peripherals
}

#[cfg(feature = "defmt")]
defmt::timestamp!("{=u64:us}", {
    // NOTE(interrupt-safe) single instruction volatile read operation
    crate::time::Instant::now()
        .duration_since_epoch()
        .as_micros()
});
