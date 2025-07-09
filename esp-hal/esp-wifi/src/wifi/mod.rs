//! WiFi

// 4
mod internal;
pub(crate) mod os_adapter;

// 7
use alloc::string::String;
use core::{marker::PhantomData, ptr::addr_of};

// 17
use enumset::{EnumSet, EnumSetType};

// 54
use num_derive::FromPrimitive;

// 61
use smoltcp::phy::{Device, DeviceCapabilities};

// 64
use crate::{common_adapter::read_mac, esp_wifi_result, EspWifiController};

// 73
const MTU: usize = crate::CONFIG.mtu;

// 86
use crate::binary::include::{
    esp_wifi_init_internal, esp_wifi_set_mode, g_wifi_default_wpa_crypto_funcs,
    wifi_mode_t_WIFI_MODE_AP, wifi_mode_t_WIFI_MODE_APSTA, wifi_mode_t_WIFI_MODE_NULL,
    wifi_mode_t_WIFI_MODE_STA,
};

/// Supported Wi-Fi authentication methods.
#[derive(EnumSetType, Debug, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Default)]
#[allow(clippy::upper_case_acronyms)] // FIXME
                                      // 146
pub enum AuthMethod {
    /// No authentication (open network).
    None,

    /// Wired Equivalent Privacy (WEP) authentication.
    WEP,

    /// Wi-Fi Protected Access (WPA) authentication.
    WPA,

    /// Wi-Fi Protected Access 2 (WPA2) Personal authentication (default).
    #[default]
    WPA2Personal,

    /// WPA/WPA2 Personal authentication (supports both).
    WPAWPA2Personal,

    /// WPA2 Enterprise authentication.
    WPA2Enterprise,

    /// WPA3 Personal authentication.
    WPA3Personal,

    /// WPA2/WPA3 Personal authentication (supports both).
    WPA2WPA3Personal,

    /// WLAN Authentication and Privacy Infrastructure (WAPI).
    WAPIPersonal,
}

/// Supported Wi-Fi protocols.
#[derive(EnumSetType, Debug, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Default)]
// 181
pub enum Protocol {
    /// 802.11b protocol.
    P802D11B,

    /// 802.11b/g protocol.
    P802D11BG,

    /// 802.11b/g/n protocol (default).
    #[default]
    P802D11BGN,

    /// 802.11b/g/n long-range (LR) protocol.
    P802D11BGNLR,

    /// 802.11 long-range (LR) protocol.
    P802D11LR,

    /// 802.11b/g/n/ax protocol.
    P802D11BGNAX,
}

/// Configuration for a Wi-Fi access point.
#[derive(Clone, PartialEq, Eq)]
// 249
pub struct AccessPointConfiguration {
    /// The SSID of the access point.
    pub ssid: String,

    /// Whether the SSID is hidden or visible.
    pub ssid_hidden: bool,

    /// The channel the access point will operate on.
    pub channel: u8,

    /// The secondary channel configuration.
    pub secondary_channel: Option<u8>,

    /// The set of protocols supported by the access point.
    pub protocols: EnumSet<Protocol>,

    /// The authentication method to be used by the access point.
    pub auth_method: AuthMethod,

    /// The password for securing the access point (if applicable).
    pub password: String,

    /// The maximum number of connections allowed on the access point.
    pub max_connections: u16,
}

// 275
impl AccessPointConfiguration {
    fn validate(&self) -> Result<(), WifiError> {
        if self.ssid.len() > 32 {
            return Err(WifiError::InvalidArguments);
        }

        if self.password.len() > 64 {
            return Err(WifiError::InvalidArguments);
        }

        Ok(())
    }
}

// 289
impl Default for AccessPointConfiguration {
    fn default() -> Self {
        Self {
            ssid: String::new(),
            ssid_hidden: false,
            channel: 1,
            secondary_channel: None,
            protocols: (Protocol::P802D11B | Protocol::P802D11BG | Protocol::P802D11BGN),
            auth_method: AuthMethod::None,
            password: String::new(),
            max_connections: 255,
        }
    }
}

// 304
impl core::fmt::Debug for AccessPointConfiguration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AccessPointConfiguration")
            .field("ssid", &self.ssid)
            .field("ssid_hidden", &self.ssid_hidden)
            .field("channel", &self.channel)
            .field("secondary_channel", &self.secondary_channel)
            .field("protocols", &self.protocols)
            .field("auth_method", &self.auth_method)
            .field("password", &"**REDACTED**")
            .field("max_connections", &self.max_connections)
            .finish()
    }
}
#[cfg(feature = "defmt")]
// 320
impl defmt::Format for AccessPointConfiguration {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Default)]
        pub struct ProtocolSet(EnumSet<Protocol>);

        #[cfg(feature = "defmt")]
        impl defmt::Format for ProtocolSet {
            fn format(&self, fmt: defmt::Formatter<'_>) {
                for (i, p) in self.0.into_iter().enumerate() {
                    if i > 0 {
                        defmt::write!(fmt, " ");
                    }
                    defmt::write!(fmt, "{}", p);
                }
            }
        }

        let protocol_set = ProtocolSet(self.protocols);

        defmt::write!(
            fmt,
            "AccessPointConfiguration {{\
            ssid: {}, \
            ssid_hidden: {}, \
            channel: {}, \
            secondary_channel: {}, \
            protocols: {}, \
            auth_method: {}, \
            password: **REDACTED**, \
            max_connections: {}, \
            }}",
            self.ssid.as_str(),
            self.ssid_hidden,
            self.channel,
            self.secondary_channel,
            protocol_set,
            self.auth_method,
            self.max_connections
        );
    }
}

/// Client configuration for a Wi-Fi connection.
#[derive(Clone, PartialEq, Eq, Default)]
// 366
pub struct ClientConfiguration {
    /// The SSID of the Wi-Fi network.
    pub ssid: String,

    /// The BSSID (MAC address) of the client.
    pub bssid: Option<[u8; 6]>,

    // pub protocol: Protocol,
    /// The authentication method for the Wi-Fi connection.
    pub auth_method: AuthMethod,

    /// The password for the Wi-Fi connection.
    pub password: String,

    /// The Wi-Fi channel to connect to.
    pub channel: Option<u8>,
}

// 384
impl ClientConfiguration {
    fn validate(&self) -> Result<(), WifiError> {
        if self.ssid.len() > 32 {
            return Err(WifiError::InvalidArguments);
        }

        if self.password.len() > 64 {
            return Err(WifiError::InvalidArguments);
        }

        Ok(())
    }
}

// 398
impl core::fmt::Debug for ClientConfiguration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ClientConfiguration")
            .field("ssid", &self.ssid)
            .field("bssid", &self.bssid)
            .field("auth_method", &self.auth_method)
            .field("password", &"**REDACTED**")
            .field("channel", &self.channel)
            .finish()
    }
}

#[cfg(feature = "defmt")]
// 411
impl defmt::Format for ClientConfiguration {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(
            fmt,
            "ClientConfiguration {{\
            ssid: {}, \
            bssid: {:?}, \
            auth_method: {:?}, \
            password: **REDACTED**, \
            channel: {:?}, \
            }}",
            self.ssid.as_str(),
            self.bssid,
            self.auth_method,
            self.channel
        )
    }
}

/// Configuration of Wi-Fi operation mode.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 667
pub enum Configuration {
    /// No configuration (default).
    #[default]
    None,

    /// Client-only configuration.
    Client(ClientConfiguration),

    /// Access point-only configuration.
    AccessPoint(AccessPointConfiguration),

    /// Simultaneous client and access point configuration.
    Mixed(ClientConfiguration, AccessPointConfiguration),
    /*
    /// EAP client configuration for enterprise Wi-Fi.
    #[cfg_attr(feature = "serde", serde(skip))]
    EapClient(EapClientConfiguration),
    */
}

// 686
impl Configuration {
    // 687
    fn validate(&self) -> Result<(), WifiError> {
        match self {
            Configuration::None => Ok(()),
            Configuration::Client(client_configuration) => client_configuration.validate(),
            Configuration::AccessPoint(access_point_configuration) => {
                access_point_configuration.validate()
            }
            Configuration::Mixed(client_configuration, access_point_configuration) => {
                client_configuration.validate()?;
                access_point_configuration.validate()
            } /*
              Configuration::EapClient(eap_client_configuration) => {
                  eap_client_configuration.validate()
              }
              */
        }
    }
}

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

    /// Passed arguments are invalid.
    InvalidArguments,
}

/// Error originating from the underlying drivers
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(clippy::enum_variant_names)] // FIXME remove prefix
                                     // 1249
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

/// Get the STA MAC address
// 1318
pub fn sta_mac(mac: &mut [u8; 6]) {
    unsafe {
        read_mac(mac as *mut u8, 0);
    }
}

/// Get the AP MAC address
// 1325
pub fn ap_mac(mac: &mut [u8; 6]) {
    unsafe {
        read_mac(mac as *mut u8, 1);
    }
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

// 1701
impl WifiDeviceMode {
    fn mac_address(&self) -> [u8; 6] {
        match self {
            WifiDeviceMode::Sta => {
                let mut mac = [0; 6];
                sta_mac(&mut mac);
                mac
            }
            WifiDeviceMode::Ap => {
                let mut mac = [0; 6];
                ap_mac(&mut mac);
                mac
            }
        }
    }
}

/// A wifi device implementing smoltcp's Device trait.
// 1805
pub struct WifiDevice<'d> {
    _phantom: PhantomData<&'d ()>,
    mode: WifiDeviceMode,
}

// 1810
impl WifiDevice<'_> {
    /// Retrieves the MAC address of the Wi-Fi device.
    // 1812
    pub fn mac_address(&self) -> [u8; 6] {
        self.mode.mac_address()
    }
}

#[cfg(feature = "smoltcp")]
// 2117
impl Device for WifiDevice<'_> {
    // 2138
    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = if crate::CONFIG.max_burst_size == 0 {
            None
        } else {
            Some(crate::CONFIG.max_burst_size)
        };
        caps
    }
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

// 2494
pub(crate) mod embassy {
    use embassy_net_driver::{Capabilities, Driver, HardwareAddress};

    use super::*;

    // 2528
    impl Driver for WifiDevice<'_> {
        // 2560
        fn capabilities(&self) -> Capabilities {
            let mut caps = Capabilities::default();
            caps.max_transmission_unit = MTU;
            caps.max_burst_size = if crate::CONFIG.max_burst_size == 0 {
                None
            } else {
                Some(crate::CONFIG.max_burst_size)
            };
            caps
        }

        // 2571
        fn hardware_address(&self) -> HardwareAddress {
            HardwareAddress::Ethernet(self.mac_address())
        }
    }
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

// 2677
impl WifiController<'_> {
    /// Set the configuration.
    ///
    /// This will set the mode accordingly.
    /// You need to use Wifi::connect() for connecting to an AP.
    ///
    /// Passing [Configuration::None] will disable both, AP and STA mode.
    ///
    /// If you don't intent to use WiFi anymore at all consider tearing down
    /// WiFi completely.
    // 2854
    pub fn set_configuration(&mut self, conf: &Configuration) -> Result<(), WifiError> {
        conf.validate()?;

        let mode = match conf {
            Configuration::None => wifi_mode_t_WIFI_MODE_NULL,
            Configuration::Client(_) => wifi_mode_t_WIFI_MODE_STA,
            Configuration::AccessPoint(_) => wifi_mode_t_WIFI_MODE_AP,
            Configuration::Mixed(_, _) => wifi_mode_t_WIFI_MODE_APSTA,
            //Configuration::EapClient(_) => wifi_mode_t_WIFI_MODE_STA,
        };

        esp_wifi_result!(unsafe { esp_wifi_set_mode(mode) })?;

        Ok(())
    }
}
