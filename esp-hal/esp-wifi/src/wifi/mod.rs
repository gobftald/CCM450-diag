//! WiFi

// 4
mod internal;
pub(crate) mod os_adapter;

// 8
use core::marker::PhantomData;

// 64
use crate::EspWifiController;

// 86
use crate::binary::include::g_wifi_default_wpa_crypto_funcs;

/// Common errors.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 1116
pub enum WifiError {
    /// Unsupported operation or mode.
    Unsupported,
}

// 1331
pub(crate) fn wifi_init() -> Result<(), WifiError> {
    unsafe {
        internal::G_CONFIG.wpa_crypto_funcs = g_wifi_default_wpa_crypto_funcs;
        internal::G_CONFIG.feature_caps = internal::g_wifi_feature_caps;

        //esp_wifi_result!(esp_wifi_init_internal(addr_of!(internal::G_CONFIG)))?;

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
