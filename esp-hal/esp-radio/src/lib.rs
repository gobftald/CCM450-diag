//! ### Optimization Level
//!
//! It is necessary to build with optimization level 2 or 3 since otherwise, it
//! might not even be able to connect or advertise.
//!
//! To make it work also for your debug builds add this to your `Cargo.toml`
//! ```toml
//! [profile.dev.package.esp-wifi]
//! opt-level = 3
//! ```
//! ## Globally disable logging
//!
//! `esp-wifi` contains a lot of trace-level logging statements.
//! For maximum performance you might want to disable logging via
//! a feature flag of the `log` crate. See [documentation](https://docs.rs/log/0.4.19/log/#compile-time-filters).
//! You should set it to `release_max_level_off`.
//!
//! ### WiFi performance considerations
//!
//! The default configuration is quite conservative to reduce power and memory consumption.
//!
//! There are a number of settings which influence the general performance. Optimal settings are chip and applications specific.
//! You can get inspiration from the [ESP-IDF examples](https://github.com/espressif/esp-idf/tree/release/v5.3/examples/wifi/iperf)
//!
//! Please note that the configuration keys are usually named slightly different and not all configuration keys apply.
//!
//! By default the power-saving mode is [PowerSaveMode::None](crate::config::PowerSaveMode::None) and `ESP_WIFI_PHY_ENABLE_USB` is enabled by default.
//!
//! In addition pay attention to these configuration keys:
//! - `ESP_WIFI_RX_QUEUE_SIZE`
//! - `ESP_WIFI_TX_QUEUE_SIZE`
//! - `ESP_WIFI_MAX_BURST_SIZE`
//! # Features flags
//!
//! When using the `dump_packets` config you can use the extcap in
//! `extras/esp-wifishark` to analyze the frames in Wireshark.
//! For more information see
//! [extras/esp-wifishark/README.md](../extras/esp-wifishark/README.md)
//! ## Additional configuration
//!
//! We've exposed some configuration options that don't fit into cargo
//! features. These can be set via environment variables, or via cargo's `[env]`
//! section inside `.cargo/config.toml`.

// 83
#![no_std]
#![cfg_attr(feature = "sys-logs", feature(c_variadic))]
#![allow(static_mut_refs)]

//#[allow(unused_imports)]
//#[macro_use(info, warn, debug, trace, panic, unwrap, debug_assert, debug_assert_eq)]
//extern crate console;

// 134
extern crate alloc;

// MUST be the first module
// 137
mod fmt;

// 139
use core::marker::PhantomData;

// 142
use esp_hal::{self as hal};
// here we use the esp-radio-rtos-driver -> esp-rtos chain
use esp_radio_rtos_driver as preempt;

// 111
use esp_config::{esp_config_bool, esp_config_int};
use hal::{
    //clock::Clocks,
    clock::{Clocks, init_radio_clocks},
    rng::Rng,
    time::Rate,
};

// 153
#[cfg(feature = "wifi")]
use crate::wifi::WifiError;

// 124
//use crate::{preempt::yield_task, tasks::init_tasks};

// 178
mod binary {
    pub use esp_wifi_sys_esp32c3::*;
}

// 181
mod compat;

// 183
mod radio;
mod time;

// 186
#[cfg(feature = "wifi")]
pub mod wifi;

// 152
//pub mod config;

// 198
pub(crate) mod common_adapter;
pub(crate) mod memory_fence;

// this is just to verify that we use the correct defaults in `build.rs`
//
#[allow(clippy::assertions_on_constants)] // TODO: try assert_eq once it's usable in const context
const _: () = {
    cfg_if::cfg_if! {
        if #[cfg(not(esp32h2))] {
            core::assert!(esp_config_int!(usize, "ESP_WIFI_CONFIG_STATIC_RX_BUF_NUM") == 10);
            core::assert!(esp_config_int!(usize, "ESP_WIFI_CONFIG_DYNAMIC_RX_BUF_NUM") == 32);
            core::assert!(esp_config_int!(usize, "ESP_WIFI_CONFIG_STATIC_TX_BUF_NUM") == 0);
            core::assert!(esp_config_int!(usize, "ESP_WIFI_CONFIG_DYNAMIC_TX_BUF_NUM") == 32);
            core::assert!(esp_config_bool!("ESP_WIFI_CONFIG_AMPDU_RX_ENABLE") == true);
            core::assert!(esp_config_bool!("ESP_WIFI_CONFIG_AMPDU_TX_ENABLE") == true);
            core::assert!(esp_config_bool!("ESP_WIFI_CONFIG_AMSDU_TX_ENABLE") == false);
            core::assert!(esp_config_int!(usize, "ESP_WIFI_CONFIG_RX_BA_WIN") == 6);
        }
    };
};

/*
// 248
pub(crate) mod  {
    use portable_atomic::AtomicBool;

    // 252
    pub(crate) static WIFI: AtomicBool = AtomicBool::new(false);
}
*/

#[derive(Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 258
//pub struct EspWifiController<'d> {
pub struct Controller<'d> {
    _inner: PhantomData<&'d ()>,
}

/// A marker trait for suitable Rng sources for esp-wifi
// 336
//pub trait EspWifiRngSource: rand_core::RngCore + private::Sealed {}
pub trait EspWifiRngSource {}

// 338
impl EspWifiRngSource for Rng {}

/// Initialize for using WiFi and or BLE.
///
/// Make sure to **not** call this function while interrupts are disabled.
///
/// # The `timer` argument
///
/// The `timer` argument is a timer source that is used by the WiFi driver to
/// schedule internal tasks. The timer source can be any of the following:
///
/// - A timg `Timer` instance
/// - A systimer `Alarm` instance
/// - An `AnyTimer` instance
// 371
pub fn init<'d>() -> Result<Controller<'d>, InitializationError> {
    if is_interrupts_disabled() {
        return Err(InitializationError::InterruptsDisabled);
    }

    if !preempt::initialized() {
        return Err(InitializationError::SchedulerNotInitialized);
    }

    // A minimum clock of 80MHz is required to operate WiFi module.
    const MIN_CLOCK: Rate = Rate::from_mhz(80);
    let clocks = Clocks::get();
    if clocks.cpu_clock < MIN_CLOCK {
        return Err(InitializationError::WrongClockConfig);
    }

    crate::common_adapter::enable_wifi_power_domain();

    // no-op
    //setup_radio_isr();

    wifi_set_log_verbose();
    init_radio_clocks();

    Ok(Controller {
        _inner: PhantomData,
    })
}

// 455
#[derive(Debug)]
#[non_exhaustive]
pub struct Blocking;

// 500
#[derive(Debug)]
#[non_exhaustive]
pub struct Async(PhantomData<*const ()>);

/// Returns true if at least some interrupt levels are disabled.
// 467
fn is_interrupts_disabled() -> bool {
    #[cfg(target_arch = "xtensa")]
    return hal::xtensa_lx::interrupt::get_level() != 0
        || hal::xtensa_lx::interrupt::get_mask() == 0;

    #[cfg(target_arch = "riscv32")]
    return !hal::riscv::register::mstatus::read().mie()
        || hal::interrupt::current_runlevel() >= hal::interrupt::Priority::Priority1;
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Error which can be returned during [`init`].
#[non_exhaustive]
// 485
pub enum InitializationError {
    /// A general error occurred.
    /// The internal error code is reported.
    General(i32),
    /// An error from the Wi-Fi driver.
    #[cfg(feature = "wifi")]
    WifiError(WifiError),
    /// The current CPU clock frequency is too low.
    WrongClockConfig,
    /// Tried to initialize while interrupts are disabled.
    /// This is not supported.
    InterruptsDisabled,
    /// The scheduler is not initialized.
    SchedulerNotInitialized,
}

/// Enable verbose logging within the WiFi driver
/// Does nothing unless the `sys-logs` feature is enabled.
// 508
pub fn wifi_set_log_verbose() {
    #[cfg(all(feature = "sys-logs", not(esp32h2)))]
    unsafe {
        use crate::binary::include::{
            esp_wifi_internal_set_log_level, wifi_log_level_t_WIFI_LOG_VERBOSE,
        };

        esp_wifi_internal_set_log_level(wifi_log_level_t_WIFI_LOG_VERBOSE);
    }
}

/*
// 520
fn init_clocks() {
    let radio_clocks = unsafe { RADIO_CLK::steal() };
    RadioClockController::new(radio_clocks).init_clocks();
}
*/
