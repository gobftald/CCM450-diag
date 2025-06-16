//! You might want browse 'https://docs.espressif.com/projects/rust/'
//! the esp-wifi documentation on the esp-rs website.

#![no_std]

#[macro_use(info, unwrap)]
extern crate console;

// 108
use core::marker::PhantomData;

// 112
use esp_hal::{self as hal, peripherals::RADIO_CLK};
use hal::{
    clock::Clocks,
    rng::Rng,
    time::Rate,
    timer::{AnyTimer, PeriodicTimer},
};

#[cfg(feature = "builtin-scheduler")]
// 135
mod preempt_builtin;

// 151
pub mod config;

// 154
pub(crate) mod common_adapter;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Tunable parameters for the WiFi driver
#[allow(unused)] // currently there are no ble tunables
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
    rx_queue_size: config::ESP_WIFI_CONFIG_RX_QUEUE_SIZE,
    tx_queue_size: config::ESP_WIFI_CONFIG_TX_QUEUE_SIZE,
    static_rx_buf_num: config::ESP_WIFI_CONFIG_STATIC_RX_BUF_NUM,
    dynamic_rx_buf_num: config::ESP_WIFI_CONFIG_DYNAMIC_RX_BUF_NUM,
    static_tx_buf_num: config::ESP_WIFI_CONFIG_STATIC_TX_BUF_NUM,
    dynamic_tx_buf_num: config::ESP_WIFI_CONFIG_DYNAMIC_TX_BUF_NUM,
    ampdu_rx_enable: config::ESP_WIFI_CONFIG_AMPDU_RX_ENABLE,
    ampdu_tx_enable: config::ESP_WIFI_CONFIG_AMPDU_TX_ENABLE,
    amsdu_tx_enable: config::ESP_WIFI_CONFIG_AMSDU_TX_ENABLE,
    rx_ba_win: config::ESP_WIFI_CONFIG_RX_BA_WIN,
    max_burst_size: config::ESP_WIFI_CONFIG_MAX_BURST_SIZE,
    country_code: config::ESP_WIFI_CONFIG_COUNTRY_CODE,
    country_code_operating_class: config::ESP_WIFI_CONFIG_COUNTRY_CODE_OPERATING_CLASS,
    mtu: config::ESP_WIFI_CONFIG_MTU,
    tick_rate_hz: config::ESP_WIFI_CONFIG_TICK_RATE_HZ,
    listen_interval: config::ESP_WIFI_CONFIG_LISTEN_INTERVAL,
    beacon_timeout: config::ESP_WIFI_CONFIG_BEACON_TIMEOUT,
    ap_beacon_timeout: config::ESP_WIFI_CONFIG_AP_BEACON_TIMEOUT,
    failure_retry_cnt: config::ESP_WIFI_CONFIG_FAILURE_RETRY_CNT,
    scan_method: config::ESP_WIFI_CONFIG_SCAN_METHOD,
};

// 246
//type TimeBase = PeriodicTimer<'static, Blocking>
type TimeBase = PeriodicTimer<'static>;

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
    T: esp_hal::timer::IntoAnyTimer,
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
pub trait EspWifiRngSource: rand_core::RngCore {}

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
    _radio_clocks: RADIO_CLK<'d>,
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

    Ok(EspWifiController {
        _inner: PhantomData,
    })
}

/// Returns true if at least some interrupt levels are disabled.
// 391
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
