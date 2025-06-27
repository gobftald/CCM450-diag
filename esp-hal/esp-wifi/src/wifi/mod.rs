//! WiFi

// 4
mod internal;
pub(crate) mod os_adapter;

// 8
use core::{marker::PhantomData, ptr::addr_of};

// 54
use num_derive::FromPrimitive;

// 64
use crate::{EspWifiController, esp_wifi_result};

// 86
use crate::binary::include::{esp_wifi_init_internal, g_wifi_default_wpa_crypto_funcs};

/// Common errors.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 1116
pub enum WifiError {
    /// Internal Wi-Fi error.
    InternalError(InternalWifiError),

    /// Unsupported operation or mode.
    Unsupported,
}

/// Error originating from the underlying drivers
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(clippy::enum_variant_names)] // FIXME remove prefix
pub enum InternalWifiError {
    /// Out of memory
    EspErrNoMem = 0x101,

    /// Invalid argument
    EspErrInvalidArg = 0x102,

    /// WiFi driver was not installed by esp_wifi_init
    EspErrWifiNotInit = 0x3001,

    /// WiFi driver was not started by esp_wifi_start
    EspErrWifiNotStarted = 0x3002,

    /// WiFi driver was not stopped by esp_wifi_stop
    EspErrWifiNotStopped = 0x3003,

    /// WiFi interface error
    EspErrWifiIf = 0x3004,

    /// WiFi mode error
    EspErrWifiMode = 0x3005,

    /// WiFi internal state error
    EspErrWifiState = 0x3006,

    /// WiFi internal control block of station or soft-AP error
    EspErrWifiConn = 0x3007,

    /// WiFi internal NVS module error
    EspErrWifiNvs = 0x3008,

    /// MAC address is invalid
    EspErrWifiMac = 0x3009,

    /// SSID is invalid
    EspErrWifiSsid = 0x300A,

    /// Password is invalid
    EspErrWifiPassword = 0x300B,

    /// Timeout error
    EspErrWifiTimeout = 0x300C,

    /// WiFi is in sleep state(RF closed) and wakeup fail
    EspErrWifiWakeFail = 0x300D,

    /// The caller would block
    EspErrWifiWouldBlock = 0x300E,

    /// Station still in disconnect status
    EspErrWifiNotConnect = 0x300F,

    /// Failed to post the event to WiFi task
    EspErrWifiPost = 0x3012,

    /// Invalid WiFi state when init/deinit is called
    EspErrWifiInitState = 0x3013,

    /// Returned when WiFi is stopping
    EspErrWifiStopState = 0x3014,

    /// The WiFi connection is not associated
    EspErrWifiNotAssoc = 0x3015,

    /// The WiFi TX is disallowed
    EspErrWifiTxDisallow = 0x3016,
}

// 1331
pub(crate) fn wifi_init() -> Result<(), WifiError> {
    unsafe {
        internal::G_CONFIG.wpa_crypto_funcs = g_wifi_default_wpa_crypto_funcs;
        internal::G_CONFIG.feature_caps = internal::g_wifi_feature_caps;

        esp_wifi_result!(esp_wifi_init_internal(addr_of!(internal::G_CONFIG)))?;

        Ok(())
    }
}

/// Provides methods for retrieving the Wi-Fi mode and MAC address.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 1696
pub enum WifiDeviceMode {
    Sta,
    Ap,
}

/// A wifi device implementing smoltcp's Device trait.
// 1805
pub struct WifiDevice<'d> {
    _phantom: PhantomData<&'d ()>,
    mode: WifiDeviceMode,
}

#[macro_export]
// 2479
macro_rules! esp_wifi_result {
    ($value:expr) => {{
        use num_traits::FromPrimitive;
        let result = $value;
        if result != esp_wifi_sys::include::ESP_OK as i32 {
            warn!("{} returned an error: {}", stringify!($value), result);
            Err(WifiError::InternalError(unwrap!(FromPrimitive::from_i32(
                result
            ))))
        } else {
            Ok::<(), WifiError>(())
        }
    }};
}

#[non_exhaustive]
// 2598
pub struct Interfaces<'d> {
    pub sta: WifiDevice<'d>,
    pub ap: WifiDevice<'d>,
    //#[cfg(feature = "esp-now")]
    //pub esp_now: crate::esp_now::EspNow<'d>,
    //#[cfg(feature = "sniffer")]
    //pub sniffer: Sniffer,
}

/// Create a WiFi controller and it's associated interfaces.
///
/// Dropping the controller will deinitialize / stop WiFi.
///
/// Make sure to **not** call this function while interrupts are disabled.
// 2612
pub fn new<'d>(
    _inited: &'d EspWifiController<'d>,
    _device: crate::hal::peripherals::WIFI<'d>,
) -> Result<(WifiController<'d>, Interfaces<'d>), WifiError> {
    if crate::is_interrupts_disabled() {
        return Err(WifiError::Unsupported);
    }

    let mut controller = WifiController {
        _phantom: Default::default(),
    };

    crate::wifi::wifi_init()?;

    Ok((
        controller,
        Interfaces {
            sta: WifiDevice {
                _phantom: Default::default(),
                mode: WifiDeviceMode::Sta,
            },
            ap: WifiDevice {
                _phantom: Default::default(),
                mode: WifiDeviceMode::Ap,
            },
            //#[cfg(feature = "esp-now")]
            //esp_now: crate::esp_now::EspNow::new_internal(),
            //#[cfg(feature = "sniffer")]
            //sniffer: Sniffer::new(),
        },
    ))
}

#[non_exhaustive]
// 2664
pub struct WifiController<'d> {
    _phantom: PhantomData<&'d ()>,
}
