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

// MUST be the first module
mod fmt;

//#[allow(unused_imports)]
//#[macro_use(info, warn, debug, trace, panic, unwrap, debug_assert, debug_assert_eq)]
//extern crate console;

// 103
extern crate alloc;

// 108
use core::marker::PhantomData;

// 111
use esp_config::{esp_config_bool, esp_config_int, esp_config_str};
use esp_hal::{self as hal /*, clock::RadioClockController, peripherals::RADIO_CLK*/};
use hal::{
    //clock::Clocks,
    clock::{init_radio_clocks, Clocks},
    rng::Rng,
    time::Rate,
    timer::{AnyTimer, PeriodicTimer},
};

// 124
use crate::{preempt::yield_task, tasks::init_tasks};

// 130
mod binary {
    pub use esp_wifi_sys::*;
}

// 133
mod compat;

#[cfg(feature = "builtin-scheduler")]
// 136
mod preempt_builtin;

// 138
pub mod preempt;

// 140
mod radio;
mod time;

#[cfg(feature = "wifi")]
// 144
pub mod wifi;

// 152
pub mod config;

// 154
pub(crate) mod common_adapter;

// 157
pub mod tasks;

// 159
pub(crate) mod memory_fence;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Tunable parameters for the WiFi driver
// currently there are no ble tunables
#[allow(unused)]
// 182
struct Config {
    rx_queue_size: usize,
    tx_queue_size: usize,
    static_rx_buf_num: usize,
    dynamic_rx_buf_num: usize,
    static_tx_buf_num: usize,
    dynamic_tx_buf_num: usize,
    ampdu_rx_enable: bool,
    ampdu_tx_enable: bool,
    amsdu_tx_enable: bool,
    rx_ba_win: usize,
    max_burst_size: usize,
    country_code: &'static str,
    country_code_operating_class: u8,
    mtu: usize,
    tick_rate_hz: u32,
    listen_interval: u16,
    beacon_timeout: u16,
    ap_beacon_timeout: u16,
    failure_retry_cnt: u8,
    scan_method: u32,
}

// 205
pub(crate) const CONFIG: config::EspWifiConfig = config::EspWifiConfig {
    rx_queue_size: esp_config_int!(usize, "ESP_WIFI_CONFIG_RX_QUEUE_SIZE"),
    tx_queue_size: esp_config_int!(usize, "ESP_WIFI_CONFIG_TX_QUEUE_SIZE"),
    static_rx_buf_num: esp_config_int!(usize, "ESP_WIFI_CONFIG_STATIC_RX_BUF_NUM"),
    dynamic_rx_buf_num: esp_config_int!(usize, "ESP_WIFI_CONFIG_DYNAMIC_RX_BUF_NUM"),
    static_tx_buf_num: esp_config_int!(usize, "ESP_WIFI_CONFIG_STATIC_TX_BUF_NUM"),
    dynamic_tx_buf_num: esp_config_int!(usize, "ESP_WIFI_CONFIG_DYNAMIC_TX_BUF_NUM"),
    ampdu_rx_enable: esp_config_bool!("ESP_WIFI_CONFIG_AMPDU_RX_ENABLE"),
    ampdu_tx_enable: esp_config_bool!("ESP_WIFI_CONFIG_AMPDU_TX_ENABLE"),
    amsdu_tx_enable: esp_config_bool!("ESP_WIFI_CONFIG_AMSDU_TX_ENABLE"),
    rx_ba_win: esp_config_int!(usize, "ESP_WIFI_CONFIG_RX_BA_WIN"),
    max_burst_size: esp_config_int!(usize, "ESP_WIFI_CONFIG_MAX_BURST_SIZE"),
    country_code: esp_config_str!("ESP_WIFI_CONFIG_COUNTRY_CODE"),
    country_code_operating_class: esp_config_int!(
        u8,
        "ESP_WIFI_CONFIG_COUNTRY_CODE_OPERATING_CLASS"
    ),
    mtu: esp_config_int!(usize, "ESP_WIFI_CONFIG_MTU"),
    tick_rate_hz: esp_config_int!(u32, "ESP_WIFI_CONFIG_TICK_RATE_HZ"),
    listen_interval: esp_config_int!(u16, "ESP_WIFI_CONFIG_LISTEN_INTERVAL"),
    beacon_timeout: esp_config_int!(u16, "ESP_WIFI_CONFIG_BEACON_TIMEOUT"),
    ap_beacon_timeout: esp_config_int!(u16, "ESP_WIFI_CONFIG_AP_BEACON_TIMEOUT"),
    failure_retry_cnt: esp_config_int!(u8, "ESP_WIFI_CONFIG_FAILURE_RETRY_CNT"),
    scan_method: esp_config_int!(u32, "ESP_WIFI_CONFIG_SCAN_METHOD"),
};

// Validate the configuration at compile time
#[allow(clippy::assertions_on_constants)]
// 233
const _: () = {
    // We explicitely use `core` assert here because this evaluation happens at
    // compile time and won't bloat the binary
    core::assert!(
        CONFIG.rx_ba_win < CONFIG.dynamic_rx_buf_num,
        "WiFi configuration check: rx_ba_win should not be larger than dynamic_rx_buf_num!"
    );
    core::assert!(
        CONFIG.rx_ba_win < (CONFIG.static_rx_buf_num * 2),
        "WiFi configuration check: rx_ba_win should not be larger than double of the static_rx_buf_num!"
    );
};

// 246
//type TimeBase = PeriodicTimer<'static, Blocking>
type TimeBase = PeriodicTimer<'static>;

// 248
pub(crate) mod flags {
    use portable_atomic::AtomicBool;

    // 252
    pub(crate) static WIFI: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 258
pub struct EspWifiController<'d> {
    _inner: PhantomData<&'d ()>,
}

/// A trait to allow better UX for initializing esp-wifi.
///
/// This trait is meant to be used only for the `init` function.
/// Calling `timers()` multiple times may panic.
// 302
//pub trait EspWifiTimerSource: private::Sealed {
pub trait EspWifiTimerSource {
    /// Returns the timer source.
    ///
    /// # Safety
    ///
    /// It is UB to call this method outside of [`init`].
    unsafe fn timer(self) -> TimeBase;
}

// 317
impl<T> EspWifiTimerSource for T
where
    //T: esp_hal::timer::IntoAnyTimer + private::Sealed,
    T: esp_hal::timer::any::Degrade,
{
    // 321
    unsafe fn timer(self) -> TimeBase {
        let any_timer: AnyTimer<'_> = self.degrade();
        let any_timer: AnyTimer<'static> = unsafe {
            // Safety: this method is only safe to be called from within `init`.
            // This 'static lifetime is a fake one, the timer is only used for the lifetime
            // of the `EspWifiController` instance. The lifetime bounds on `init` and
            // `EspWifiTimerSource` ensure that the timer is not used after the
            // `EspWifiController` is dropped.
            core::mem::transmute(any_timer)
        };
        TimeBase::new(any_timer)
    }
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
pub fn init<'d>(
    timer: impl EspWifiTimerSource + 'd,
    _rng: impl EspWifiRngSource + 'd,
    //_radio_clocks: RADIO_CLK<'d>,
) -> Result<EspWifiController<'d>, InitializationError> {
    if is_interrupts_disabled() {
        return Err(InitializationError::InterruptsDisabled);
    }

    // A minimum clock of 80MHz is required to operate WiFi module.
    const MIN_CLOCK: Rate = Rate::from_mhz(80);
    let clocks = Clocks::get();
    if clocks.cpu_clock < MIN_CLOCK {
        return Err(InitializationError::WrongClockConfig);
    }

    info!("esp-wifi configuration {:?}", CONFIG);
    common_adapter::chip_specific::enable_wifi_power_domain();
    common_adapter::chip_specific::phy_mem_init();

    // no-op
    //setup_radio_isr();

    // Enable timer tick interrupt
    #[cfg(feature = "builtin-scheduler")]
    // 365
    //preempt_builtin::setup_timer(unsafe { timer.timer() });
    preempt_builtin::setup_timer(unsafe { timer.timer() });

    // This initializes the task switcher
    preempt::enable();

    init_tasks();
    yield_task(); // don't wait for the next builtin scheduler tick IRQ

    wifi_set_log_verbose();
    //init_clocks();
    init_radio_clocks();

    Ok(EspWifiController {
        _inner: PhantomData,
    })
}

/// Returns true if at least some interrupt levels are disabled.
// 467
fn is_interrupts_disabled() -> bool {
    #[cfg(target_arch = "xtensa")]
    return hal::xtensa_lx::interrupt::get_level() != 0
        || hal::xtensa_lx::interrupt::get_mask() == 0;

    #[cfg(target_arch = "riscv32")]
    return !hal::riscv::register::mstatus::read().mie();
    // we don't use "runlevel" yet
    //|| hal::interrupt::current_runlevel() >= hal::interrupt::Priority::Priority1;
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Error which can be returned during [`init`].
#[non_exhaustive]
// 485
pub enum InitializationError {
    /// The current CPU clock frequency is too low.
    WrongClockConfig,
    /// Tried to initialize while interrupts are disabled.
    /// This is not supported.
    InterruptsDisabled,
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
